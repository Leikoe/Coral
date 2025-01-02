#![deny(clippy::unwrap_used)]
#![allow(async_fn_in_trait)]
pub mod actions;
pub mod controllers;
pub mod game_controller;
pub mod league_protocols;
pub mod math;
pub mod net;
pub mod testing;
pub mod trajectories;
pub mod viewer;
pub mod vision;
pub mod world;

use std::{collections::HashMap, fmt::Debug, sync::LockResult, time::Duration};

use controllers::RobotController;
use league_protocols::simulation_packet::RobotFeedback;
use math::Point2;
use tokio::{
    select,
    sync::oneshot::{self, Sender},
    task::JoinHandle,
};
use tracing::{debug, info, warn};
use viewer::ViewerObject;
use vision::Vision;
use world::{AllyRobot, EnnemyRobot, RobotId, TeamColor, World};

pub const CONTROL_PERIOD: Duration = Duration::from_millis(10);
pub const DETECTION_SCALING_FACTOR: f64 = 1000.;

pub trait IgnoreMutexErr<T> {
    fn unwrap_ignore_poison(self) -> T;
}

impl<T> IgnoreMutexErr<T> for LockResult<T> {
    fn unwrap_ignore_poison(self) -> T {
        match self {
            Ok(r) => r,
            Err(poisoned) => {
                // Handle mutex poisoning
                let guard = poisoned.into_inner();
                warn!("mutex was poisoned, recovering from mutex poisoning");
                guard
            }
        }
    }
}

async fn control_loop<
    E: Debug,
    C: RobotController<HashMap<RobotId, RobotFeedback>, E> + Send + 'static,
>(
    world: World,
    controller: &mut C,
) {
    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    loop {
        interval.tick().await; // first tick ticks immediately that's why it's at the beginning

        // ugly hack, could have been a `impl Iterator<Item &Robot>` if I was better at rust :/
        let robots = world
            .team
            .lock()
            .unwrap_ignore_poison()
            .values()
            .cloned()
            .collect::<Vec<AllyRobot>>();
        let feedback_per_robot = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands to robots");
        for (rid, feedback) in feedback_per_robot {
            if let Some(robot) = world.team.lock().unwrap_ignore_poison().get_mut(&rid) {
                robot.set_has_ball(feedback.dribbler_ball_contact());
            }
        }
    }
}

pub fn launch_control_thread<E: Debug>(
    world: World,
    mut controller: impl RobotController<HashMap<RobotId, RobotFeedback>, E> + Send + 'static,
) -> (Sender<()>, JoinHandle<()>) {
    let (stop_sender, stop_receiver) = oneshot::channel();
    let handle = tokio::spawn(async move {
        select! {
            _ = control_loop(world, &mut controller) => {}
            _ = stop_receiver => {
                info!("control thread received stop signal")
            }
        };

        controller.close().await.expect("couldn't close controller");
    });
    (stop_sender, handle)
}

pub async fn update_world_with_vision_forever(mut world: World, real: bool) {
    let mut vision = Vision::new(None, None, real);
    let mut ball_drawing = viewer::start_drawing(ViewerObject::Point {
        color: "orange",
        pos: world.ball.get_pos(),
    });
    let update_notifier = world.get_update_notifier();
    loop {
        while let Ok(packet) = vision.receive().await {
            let mut ally_team = world.team.lock().unwrap_ignore_poison();
            let mut ennemy_team = world.ennemies.lock().unwrap_ignore_poison();
            let ball = world.ball.clone();
            if let Some(detection) = packet.detection {
                // println!("NEW CAM PACKET!");
                let detection_time = detection.t_capture;
                if let Some(ball_detection) = detection.balls.first() {
                    let detected_pos = Point2::new(
                        ball_detection.x as f64 / DETECTION_SCALING_FACTOR,
                        ball_detection.y as f64 / DETECTION_SCALING_FACTOR,
                    );
                    if let Some(last_t) = ball.get_last_update() {
                        let dt = detection_time - last_t;
                        ball.set_vel((detected_pos - ball.get_pos()) / dt);
                    }
                    // println!("{:?}", detected_pos);
                    ball.set_pos(detected_pos);
                    ball_drawing.update(ViewerObject::Point {
                        color: "orange",
                        pos: detected_pos,
                    });
                }

                let (allies, ennemies) = match world.team_color {
                    TeamColor::Blue => (detection.robots_blue, detection.robots_yellow),
                    TeamColor::Yellow => (detection.robots_yellow, detection.robots_blue),
                };
                for ally_detection in allies {
                    let rid = ally_detection.robot_id() as u8;
                    if ally_team.get_mut(&rid).is_none() {
                        debug!("added ally {} to the team!", rid);
                        let r = AllyRobot::default_with_id(rid, world.team_color);
                        ally_team.insert(rid, r);
                    }
                    // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                    let r = ally_team
                        .get_mut(&rid)
                        .expect("pre inserted robot MUST be present");
                    r.update_from_packet(ally_detection, &ball, detection_time);
                }

                for ennemy_detection in ennemies {
                    let rid = ennemy_detection.robot_id() as u8;
                    if ennemy_team.get_mut(&rid).is_none() {
                        debug!("added ennemy {} to the ennemies!", rid);
                        let r = EnnemyRobot::default_with_id(rid, world.team_color.opposite());
                        ennemy_team.insert(rid, r);
                    }
                    // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                    let r = ennemy_team
                        .get_mut(&rid)
                        .expect("pre inserted robot MUST be present");
                    r.update_from_packet(ennemy_detection, &ball, detection_time);
                }
                update_notifier.notify_waiters();
            }
            if let Some(geometry) = packet.geometry {
                world.field.update_from_packet(geometry.field);
            }
        }
    }
}
