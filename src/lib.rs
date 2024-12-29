#[allow(async_fn_in_trait)]
extern crate nalgebra as na;

pub mod actions;
pub mod controllers;
pub mod game_controller;
pub mod league_protocols;
pub mod math;
pub mod net;
pub mod posvelacc;
pub mod trajectories;
pub mod viewer;
pub mod vision;
pub mod world;

use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

use controllers::RobotController;
use league_protocols::simulation_packet::RobotFeedback;
use tokio::{select, sync::Notify, task::JoinHandle};
use world::{AllyRobot, RobotId, World};

pub const CONTROL_PERIOD: Duration = Duration::from_millis(10);
pub const DETECTION_SCALING_FACTOR: f64 = 1000.;

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
            .unwrap()
            .values()
            .map(|r| r.clone())
            .collect::<Vec<AllyRobot>>();
        let feedback_per_robot = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands to robots");
        for (rid, feedback) in feedback_per_robot {
            if let Some(robot) = world.team.lock().unwrap().get_mut(&rid) {
                robot.set_has_ball(feedback.dribbler_ball_contact());
            }
        }
    }
}

pub fn launch_control_thread<E: Debug>(
    world: World,
    mut controller: impl RobotController<HashMap<RobotId, RobotFeedback>, E> + Send + 'static,
) -> (Arc<Notify>, JoinHandle<()>) {
    let notifier = Arc::new(tokio::sync::Notify::new());
    let notifier_clone = notifier.clone();
    let handle = tokio::spawn(async move {
        select! {
            _ = control_loop(world, &mut controller) => {}
            _ = notifier_clone.notified() => {}
        };

        controller.close().await.expect("couldn't close controller");
    });
    (notifier, handle)
}
