#[allow(async_fn_in_trait)]
pub mod actions;
pub mod controllers;
pub mod game_controller;
pub mod league_protocols;
pub mod math;
pub mod net;
pub mod trajectories;
pub mod vision;
pub mod world;

use std::{
    fmt::Debug,
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use controllers::RobotController;
use math::Point2;
use tokio::{select, sync::Notify, task::JoinHandle, time::Interval};
use vision::Vision;
use world::{AllyRobot, EnnemyRobot, TeamColor, World};

pub const CONTROL_PERIOD: Duration = Duration::from_millis(10);
const DETECTION_SCALING_FACTOR: f32 = 1000.;

async fn control_loop<T, E: Debug, C: RobotController<T, E> + Send + 'static>(
    mut world: World,
    mut vision: Vision,
    side: TeamColor,
    mut interval: Interval,
    controller: &mut C,
) {
    loop {
        interval.tick().await; // first tick ticks immediately that's why it's at the beginning
                               // let mut pending_packets_count = 0;
        let pending_packets_iterator = vision.take_pending_packets().await;
        {
            let mut ally_team = world.team.lock().unwrap();
            let mut ennemy_team = world.ennemies.lock().unwrap();
            let ball = world.ball.clone();
            for packet in pending_packets_iterator {
                if let Some(detection) = packet.detection {
                    let detection_time =
                        world.get_creation_time() + Duration::from_secs_f64(detection.t_capture);
                    if let Some(ball_detection) = detection.balls.get(0) {
                        ball.set_pos(Point2::new(
                            ball_detection.x / DETECTION_SCALING_FACTOR,
                            ball_detection.y / DETECTION_SCALING_FACTOR,
                        ));
                    }

                    // TODO: handle ennemies
                    let (allies, ennemies) = match side {
                        TeamColor::Blue => (detection.robots_blue, detection.robots_yellow),
                        TeamColor::Yellow => (detection.robots_yellow, detection.robots_blue),
                    };
                    for ally_detection in allies {
                        let rid = ally_detection.robot_id() as u8;
                        if ally_team.get_mut(&rid).is_none() {
                            println!("[DEBUG] added ally {} to the team!", rid);
                            let r = AllyRobot::default_with_id(rid);
                            ally_team.insert(rid, r);
                        }
                        // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                        let r = ally_team.get_mut(&rid).unwrap();
                        r.update_from_packet(ally_detection, &ball, detection_time);
                    }

                    for ennemy_detection in ennemies {
                        let rid = ennemy_detection.robot_id() as u8;
                        if ennemy_team.get_mut(&rid).is_none() {
                            println!("[DEBUG] added ennemy {} to the ennemies!", rid);
                            let r = EnnemyRobot::default_with_id(rid);
                            ennemy_team.insert(rid, r);
                        }
                        // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                        let r = ennemy_team.get_mut(&rid).unwrap();
                        r.update_from_packet(ennemy_detection, &ball, detection_time);
                    }
                }
                if let Some(geometry) = packet.geometry {
                    world.field.update_from_packet(geometry.field);
                }
                // pending_packets_count += 1;
            }
        }

        // println!(
        //     "[TRACE] {} packets were pending, they were applied!",
        //     pending_packets_count
        // );

        // println!("[DEBUG] world state");
        // println!(
        //     "\tball pos: {:?}",
        //     world.lock().unwrap().ball.get_reactive()
        // );
        // for r in world.lock().unwrap().team.values() {
        //     println!(
        //         "\trobot {} | is_dribbling: {} | pos: {:?} | orientation: {}",
        //         r.get_id(),
        //         r.should_dribble(),
        //         r.get_reactive(),
        //         r.get_orientation()
        //     );
        // }

        // ugly hack, could have been a `impl Iterator<Item &Robot>` if I was better at rust :/
        let robots = world
            .team
            .lock()
            .unwrap()
            .values()
            .map(|r| r.clone())
            .collect::<Vec<AllyRobot>>();
        let _ = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands to robots");
    }
}

pub fn launch_control_thread<T, E: Debug>(
    world: World,
    custom_vision_ip: Option<Ipv4Addr>, // the vision ip string stays valid for the whole app's duration
    custom_vision_port: Option<u16>,
    real: bool,
    side: TeamColor,
    mut controller: impl RobotController<T, E> + Send + 'static,
) -> (Arc<Notify>, JoinHandle<()>) {
    let notifier = Arc::new(tokio::sync::Notify::new());
    let notifier_clone = notifier.clone();
    let interval = tokio::time::interval(CONTROL_PERIOD);
    let handle = tokio::spawn(async move {
        let vision = Vision::new(custom_vision_ip, custom_vision_port, real);

        select! {
            _ = control_loop(world, vision, side, interval, &mut controller) => {}
            _ = notifier_clone.notified() => {}
        };

        controller.close().await.expect("couldn't close controller");
    });
    (notifier, handle)
}
