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
pub const DETECTION_SCALING_FACTOR: f32 = 1000.;

async fn control_loop<T, E: Debug, C: RobotController<T, E> + Send + 'static>(
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
        let _ = controller
            .send_proper_command_for(robots.into_iter())
            .await
            .expect("couldn't send commands to robots");
    }
}

pub fn launch_control_thread<T, E: Debug>(
    world: World,
    mut controller: impl RobotController<T, E> + Send + 'static,
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
