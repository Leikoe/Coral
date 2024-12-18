#[allow(async_fn_in_trait)]
pub mod actions;
pub mod controllers;
pub mod league_protocols;
pub mod math;
pub mod net;
pub mod vision;
pub mod world;

use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Duration,
};

use controllers::RobotController;
use math::{angle_difference, Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext};
use tokio::{select, sync::Notify, task::JoinHandle, time::Interval};
use vision::Vision;
use world::{AllyRobot, TeamColor, World};

pub const CONTROL_PERIOD: Duration = Duration::from_millis(50);

async fn control_loop<T, E: Debug, C: RobotController<T, E> + Send + 'static>(
    world: Arc<Mutex<World>>,
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
            let mut w = world.lock().unwrap();
            let ball = w.ball.clone();
            for packet in pending_packets_iterator {
                if let Some(detection) = packet.detection {
                    if let Some(ball_detection) = detection.balls.get(0) {
                        ball.set_pos(Point2::new(
                            ball_detection.x / 1000.,
                            ball_detection.y / 1000.,
                        ));
                    }

                    // TODO: handle ennemies
                    let (allies, _ennemies) = match side {
                        TeamColor::Blue => (detection.robots_blue, detection.robots_yellow),
                        TeamColor::Yellow => (detection.robots_yellow, detection.robots_blue),
                    };
                    for ally_detection in allies {
                        let rid = ally_detection.robot_id() as u8;
                        let detected_pos =
                            Point2::new(ally_detection.x / 1000., ally_detection.y / 1000.);
                        let detected_orientation = ally_detection.orientation();
                        if w.team.get_mut(&rid).is_none() {
                            println!("[DEBUG] ally {} was added to team!", rid);
                            let r = AllyRobot::new(rid, detected_pos, detected_orientation);
                            w.team.insert(rid, r);
                        }

                        {
                            // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                            let r = w.team.get_mut(&rid).unwrap();
                            r.set_orientation(detected_orientation);
                            r.set_pos(detected_pos);
                        }
                        let has_ball = {
                            let r = w.team.get(&rid).unwrap();
                            let r_to_ball = r.to(&ball);
                            let is_facing_ball = angle_difference(
                                r_to_ball.angle() as f64,
                                r.get_orientation() as f64,
                            )
                            .abs()
                                < 20.;
                            is_facing_ball && (r_to_ball.norm() < 0.11) // TODO: stop the magic
                        };

                        // SAFETY: if the robot wasn't present, we inserted it & we hold the lock. Therefore it MUST be in the map
                        let r = w.team.get_mut(&rid).unwrap();
                        // if rid == 0 {
                        //     dbg!(r_to_ball.angle());
                        //     dbg!(r.get_orientation());
                        //     dbg!(r_to_ball.norm());
                        //     dbg!(has_ball);
                        // }
                        r.set_has_ball(has_ball);
                    }
                }

                // pending_packets_count += 1;
            }
        }

        // println!(
        //     "[TRACE] {} packets were pending, they were applied!",
        //     pending_packets_count
        // );

        println!("[DEBUG] world state");
        println!(
            "\tball pos: {:?}",
            world.lock().unwrap().ball.get_reactive()
        );
        for r in world.lock().unwrap().team.values() {
            println!(
                "\trobot {} | is_dribbling: {} | pos: {:?} | orientation: {}",
                r.get_id(),
                r.should_dribble(),
                r.get_reactive(),
                r.get_orientation()
            );
        }

        // ugly hack, could have been a `impl Iterator<Item &Robot>` if I was better at rust :/
        let robots = world
            .lock()
            .unwrap()
            .team
            .values()
            .map(|r| r.clone())
            .collect::<Vec<AllyRobot>>();
        let _ = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands");
    }
}

pub fn launch_control_thread<T, E: Debug>(
    world: Arc<Mutex<World>>,
    vision_address: &'static str, // the vision ip string stays valid for the whole app's duration
    vision_port: Option<u16>,
    real: bool,
    side: TeamColor,
    mut controller: impl RobotController<T, E> + Send + 'static,
) -> (Arc<Notify>, JoinHandle<()>) {
    let notifier = Arc::new(tokio::sync::Notify::new());
    let notifier_clone = notifier.clone();
    let interval = tokio::time::interval(CONTROL_PERIOD);
    let handle = tokio::spawn(async move {
        let vision = Vision::new(vision_address, vision_port, real);

        // Robot control loop, 1Hz
        select! {
            _ = control_loop(world, vision, side, interval, &mut controller) => {}
            _ = notifier_clone.notified() => {}
        };

        controller.close().await.expect("couldn't close controller");
    });
    (notifier, handle)
}
