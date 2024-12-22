use crabe_async::{
    actions::{place_ball, strike_alone, three_attackers_attack},
    controllers::sim_controller::SimRobotController,
    launch_control_thread,
    math::{Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext, Rect, Vec2},
    trajectories::{bangbang2d::BangBang2d, Trajectory},
    world::{AvoidanceMode, Ball, TeamColor, World},
    CONTROL_PERIOD,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{join, time::sleep};

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let world = World {
        // TODO: don't assume field dims
        field: Arc::new(Mutex::new(Rect::new(
            Point2::new(-4.5, 3.),
            Point2::new(4.5, -3.),
        ))),
        ball: Ball::default(),
        team: Default::default(),
        ennemies: Default::default(),
    };

    let color = TeamColor::Blue;
    let controller = SimRobotController::new(color).await;
    let (control_loop_thread_stop_notifier, control_loop_thread_handle) =
        launch_control_thread(world.clone(), "224.5.23.2", None, false, color, controller);
    sleep(CONTROL_PERIOD * 10).await; // AWAIT ROBOTS DETECTION

    let r0 = world.team.lock().unwrap().get(&3).unwrap().clone();
    let r1 = world.team.lock().unwrap().get(&4).unwrap().clone();
    let r2 = world.team.lock().unwrap().get(&5).unwrap().clone();
    let ball = world.ball.clone();

    // shoot(&world, &r0, &ball).await;

    // place_ball(&world, &r0, &ball, &goal).await;
    // let _ = r0
    //     .goto(&world, &Point2::zero(), None, AvoidanceMode::AvoidRobots)
    //     .await;
    r0.go_get_ball(&world, &ball).await;
    three_attackers_attack(&world, &r1, &r0, &r2).await;
    let (d1, d0, d2) = (
        Point2::new(-1., 1.),
        Point2::new(-1., 0.),
        Point2::new(-1., -1.),
    );
    let _ = join!(
        r1.goto(&world, &d1, Some(0.), AvoidanceMode::AvoidRobots,),
        r0.goto(&world, &d0, Some(0.), AvoidanceMode::AvoidRobots,),
        r2.goto(&world, &d2, Some(0.), AvoidanceMode::AvoidRobots,),
    );

    sleep(Duration::from_millis(100)).await;
    control_loop_thread_stop_notifier.notify_one(); // ask for stop
    control_loop_thread_handle
        .await
        .expect("failed to stop control loop thread!"); // wait done stopping
    sleep(Duration::from_millis(100)).await;
}
