use crabe_async::{
    actions::shoot,
    controllers::sim_controller::SimRobotController,
    launch_control_thread,
    math::{Point2, Reactive, Rect, Vec2},
    trajectories::{bangbang2d::BangBang2d, Trajectory},
    world::{AvoidanceMode, Ball, TeamColor, World},
    CONTROL_PERIOD,
};
use plotters::{
    chart::{ChartBuilder, LabelAreaPosition},
    prelude::{BitMapBackend, Circle, IntoDrawingArea},
    series::LineSeries,
    style::{full_palette::GREEN, Color, BLUE, RED, WHITE},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

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
    let ball = world.ball.clone();

    // shoot(&world, &r0, &ball).await;

    // do a square
    // r0.set_target_vel(Vec2::new(1., 0.));
    // sleep(Duration::from_secs(1)).await;
    // let path = do_square_rrt(&world, &r0)
    //     .await
    //     .expect("couldn't find a path");

    let goal = Point2::new(2., 0.);
    let _ = r0
        .goto_traj(&world, &goal, None, AvoidanceMode::AvoidRobots)
        .await;

    // for i in 0.. {
    //     println!("new traj {}!", i);
    //     let traj = BangBang2d::new(r0.get_pos(), Vec2::zero(), goal, 1., 2., 0.01);
    //     let start = Instant::now();
    //     while start.elapsed().as_secs_f64() < traj.get_total_runtime() {
    //         let t = start.elapsed().as_secs_f64();
    //         let v = r0.pov_vec(traj.get_velocity(t));
    //         let p = traj.get_position(t);
    //         let p_diff = r0.pov_vec(p - r0.get_pos());
    //         if p_diff.norm() > 0.5 {
    //             println!("we fell off the traj!");
    //             // break;
    //         }
    //         r0.set_target_vel(v + p_diff * 0.5);
    //         sleep(CONTROL_PERIOD).await;
    //     }
    //     break;
    // }

    // loop {
    //     r0.set_target_vel(Vec2::zero());
    // }

    // let ennemy_goal = Point2::new(4.5, 0.);
    // for _ in 0..10 {
    //     shoot(&world, &r0, &ball, &ennemy_goal).await;
    // }

    // {
    //     // PLOT
    //     let root_area = BitMapBackend::new("plot.png", (600, 400)).into_drawing_area();
    //     root_area.fill(&WHITE).unwrap();

    //     let to_int = |f: f32| (f * 10.) as i32;

    //     let mut ctx = ChartBuilder::on(&root_area)
    //         .set_label_area_size(LabelAreaPosition::Left, 40)
    //         .set_label_area_size(LabelAreaPosition::Bottom, 40)
    //         .caption("Evitement", ("sans-serif", 40))
    //         .build_cartesian_2d(-45..45, -30..30)
    //         .unwrap();

    //     ctx.configure_mesh().draw().unwrap();

    //     ctx.draw_series(
    //         world.lock().unwrap().team.iter().map(|(id, r)| {
    //             Circle::new((to_int(r.get_pos().x), to_int(r.get_pos().y)), 5, &BLUE)
    //         }),
    //     )
    //     .unwrap();

    //     ctx.draw_series(
    //         path.iter()
    //             .map(|p| Circle::new((to_int(p.x), to_int(p.y)), 5, GREEN.filled()))
    //             .take(1),
    //     )
    //     .unwrap();

    //     // ctx.draw_series(
    //     //     vec![goal]
    //     //         .iter()
    //     //         .map(|p| Circle::new((to_int(p.x), to_int(p.y)), 5, RED.filled()))
    //     //         .take(1),
    //     // )
    //     // .unwrap();

    //     ctx.draw_series(LineSeries::new(
    //         path.iter().map(|p| (to_int(p.x), to_int(p.y))),
    //         &RED,
    //     ))
    //     .unwrap();
    // }

    sleep(Duration::from_millis(100)).await;
    control_loop_thread_stop_notifier.notify_one(); // ask for stop
    control_loop_thread_handle
        .await
        .expect("failed to stop control loop thread!"); // wait done stopping
    sleep(Duration::from_millis(100)).await;
}
