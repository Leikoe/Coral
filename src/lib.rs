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
    collections::HashMap,
    fmt::Debug,
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use controllers::RobotController;
use league_protocols::simulation_packet::RobotFeedback;
use math::Point2;
use tokio::{select, sync::Notify, task::JoinHandle, time::Interval};
use vision::Vision;
use world::{AllyRobot, EnnemyRobot, RobotId, TeamColor, World};

pub const CONTROL_PERIOD: Duration = Duration::from_millis(10);
pub const DETECTION_SCALING_FACTOR: f32 = 1000.;

async fn control_loop<
    E: Debug,
    C: RobotController<HashMap<RobotId, RobotFeedback>, E> + Send + 'static,
>(
    mut world: World,
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
        let feedback = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands to robots");
        // for v in feedback.values() {
        //     dbg!(v);
        //     if let Some(r) = world.team.lock().unwrap().get_mut(&(v.id as RobotId)) {
        //         r.set_has_ball(v.dribbler_ball_contact());
        //     }
        // }
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
