mod actions;
mod ball;
mod math;
mod robot;
mod trackable;

use actions::*;
use ball::Ball;
use math::*;
use plotters::{
    chart::{ChartBuilder, LabelAreaPosition},
    prelude::{BitMapBackend, Circle, IntoDrawingArea, TriangleMarker},
    series::LineSeries,
    style::{BLUE, RED, WHITE},
};
use robot::{Robot, RobotId};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use trackable::*;

pub const CONTROL_PERIOD: Duration = Duration::from_millis(10);

#[derive(Clone)]
pub struct World {
    pub field: Rect,
    pub ball: Ball,
    pub team: HashMap<RobotId, Robot>,
}

#[derive(Debug)]
pub struct RobotCommand {
    vel: Vec2,
    angular_vel: f32,
    dribble: bool,
    kick: bool,
}

fn take_next_commands(robots: &mut HashMap<RobotId, Robot>) -> HashMap<RobotId, RobotCommand> {
    robots
        .values()
        .map(|r| (r.get_id(), r.make_command()))
        .collect()
}

fn launch_control_thread(mut world: World) -> JoinHandle<()> {
    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    tokio::spawn(async move {
        // Robot control loop, 1Hz
        loop {
            interval.tick().await; // first tick ticks immediately that's why it's at the beginning

            // println!("[DEBUG] world state");
            // println!("\tball pos: {:?}", world.ball.get_pos());
            // for r in world.team.values() {
            //     println!(
            //         "\trobot {} | is_dribbling: {} | pos: {:?} | orientation: {}",
            //         r.get_id(),
            //         r.should_dribble(),
            //         r.get_pos(),
            //         r.get_orientation()
            //     );
            // }

            // println!("[DEBUG] sending commands!\n");
            // take the commands & apply them (simulate real robot)
            take_next_commands(&mut world.team)
                .drain()
                .for_each(|(rid, command)| {
                    if let Some(r) = world.team.get_mut(&rid) {
                        r.apply_vel(command.vel);
                        r.apply_angular_vel(command.angular_vel);
                    } else {
                        eprintln!(
                            "[WARNING] A command was sent to robot {} which isn't online!",
                            rid
                        );
                    }
                });
        }
    })
}

fn make_ball_spin(ball: Ball, timeout: Option<Duration>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let start = Instant::now();
        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while timeout.is_none() || start.elapsed() < timeout.unwrap() {
            interval.tick().await;
            let elapsed_secs = start.elapsed().as_secs_f32();
            ball.set_pos(Point2::new(elapsed_secs.cos(), elapsed_secs.sin()));
        }
    })
}

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let mut world = World {
        // TODO: don't assume field dims
        field: Rect::new(Point2::new(-3.5, 1.75), Point2::new(3.5, -1.75)),
        ball: Ball::new(Point2::new(-0.6, -0.2), Vec2::new(0.4, 0.4)),
        team: HashMap::new(),
    };
    world
        .team
        .insert(0, Robot::new(0, Point2::new(-2., 0.), 0.));
    world.team.insert(1, Robot::new(1, Point2::zero(), 0.));
    world
        .team
        .insert(2, Robot::new(2, Point2::new(0., -1.), 0.));

    // robot aliases
    let (r0, r1, r2) = (
        world.team.get(&0).unwrap(),
        world.team.get(&1).unwrap(),
        world.team.get(&2).unwrap(),
    );

    let control_loop_thread = launch_control_thread(world.clone());

    // do a square
    // do_square(r0).await;

    // // do a "three_attackers_attack" and simulate a penalty after 2s to early stop
    // let _ = tokio::time::timeout(
    //     Duration::from_millis(500),
    //     three_attackers_attack(r1, r2, r0),
    // )
    // .await;

    // // now we spin the ball and make the robot try to go get it to showcase the Trackable trait
    // make_ball_spin(world.ball.clone(), Some(Duration::from_secs(5)));
    // go_get_ball(r0, &world.ball).await;

    // // do a ball interception
    // intercept(r0, &world.ball).await;

    // showcase obstacle avoidance goto
    // teleport robots in place
    r0.debug_tp(&Point2::new(-1., 0.), None);
    r1.debug_tp(&Point2::new(0., 0.), None);
    r2.debug_tp(&Point2::new(0., -1.), None);
    let path = r0
        .goto_rrt(&world, &Point2::new(1., 0.), None)
        .await
        .unwrap();

    // PLOT
    let root_area = BitMapBackend::new("plot.png", (600, 400)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let to_int = |f: f32| (f * 10.) as i32;

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Evitement", ("sans-serif", 40))
        .build_cartesian_2d(-20..20, -15..15)
        .unwrap();

    ctx.configure_mesh().draw().unwrap();

    ctx.draw_series(
        world
            .team
            .iter()
            .map(|(id, r)| Circle::new((to_int(r.get_pos().x), to_int(r.get_pos().y)), 5, &BLUE)),
    )
    .unwrap();

    ctx.draw_series(LineSeries::new(
        path.iter().map(|p| (to_int(p.x), to_int(p.y))),
        &RED,
    ))
    .unwrap();

    control_loop_thread.abort();
}
