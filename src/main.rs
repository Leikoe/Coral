#![feature(future_join)]

mod actions;
mod ball;
mod math;
mod robot;
mod trackable;

use actions::*;
use ball::Ball;
use math::*;
use robot::{Robot, RobotId};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;
use trackable::*;

pub const CONTROL_PERIOD: Duration = Duration::from_millis(50);

#[derive(Clone)]
pub struct World {
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

            println!("[DEBUG] world state");
            println!("\tball pos: {:?}", world.ball.get_pos());
            for r in world.team.values() {
                println!(
                    "\trobot {} | is_dribbling: {} | pos: {:?} | orientation: {}",
                    r.get_id(),
                    r.should_dribble(),
                    r.get_pos(),
                    r.get_orientation()
                );
            }

            println!("[DEBUG] sending commands!\n");
            // take the commands & apply them (simulate real robot)
            take_next_commands(&mut world.team)
                .drain()
                .for_each(|(rid, command)| {
                    let r = world.team.get_mut(&rid).unwrap(); // TODO: check we have the robot to send the order to
                    r.apply_vel(command.vel);
                    r.apply_angular_vel(command.angular_vel);
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
        ball: Ball::new(Point2::new(-0.6, -0.2), Vec2::new(0.4, 0.4)),
        team: HashMap::new(),
    };
    world.team.insert(0, Robot::new(0, Point2::zero(), 0.));
    world.team.insert(1, Robot::new(1, Point2::zero(), 0.));
    world.team.insert(2, Robot::new(2, Point2::zero(), 0.));

    let control_loop_thread = launch_control_thread(world.clone());

    do_square(world.team.get(&0).unwrap()).await;
    // we simulate a penalty after 2s
    let _ = tokio::time::timeout(
        Duration::from_millis(500),
        three_attackers_attack(
            world.team.get(&1).unwrap(),
            world.team.get(&0).unwrap(),
            world.team.get(&2).unwrap(),
        ),
    )
    .await;

    // now we spin the ball and make the robot try to go get it to showcase the Trackable trait
    make_ball_spin(world.ball.clone(), Some(Duration::from_secs(5)));
    go_get_ball(world.team.get(&0).unwrap(), &world.ball).await;

    intercept(world.team.get(&0).unwrap(), &world.ball).await;

    control_loop_thread.abort();
}
