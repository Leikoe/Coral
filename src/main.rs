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
use trackable::Trackable;

const CONTROL_PERIOD: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct RobotCommand {
    vel: Vec2,
    dribble: bool,
    kick: bool,
}

fn take_next_commands(robots: &mut HashMap<RobotId, Robot>) -> HashMap<RobotId, RobotCommand> {
    robots
        .values()
        .filter_map(|r| {
            let mut next_command = r.next_command.lock().unwrap();
            if next_command.is_some() {
                Some((r.get_id(), next_command.take().unwrap()))
            } else {
                None
            }
        })
        .collect()
}

fn launch_control_thread(mut team: HashMap<RobotId, Robot>, ball: Ball) -> JoinHandle<()> {
    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    tokio::spawn(async move {
        // Robot control loop, 1Hz
        loop {
            interval.tick().await; // first tick ticks immediately that's why it's at the beginning

            println!("[DEBUG] world state");
            println!("\tball pos: {:?}", ball.get_pos());
            for r in team.values() {
                println!(
                    "\trobot {} | is_dribbling: {} | pos: {:?}",
                    r.get_id(),
                    r.is_dribbling(),
                    r.get_pos(),
                );
            }

            println!("[DEBUG] sending commands!\n");
            // take the commands & apply them (simulate real robot)
            take_next_commands(&mut team)
                .drain()
                .for_each(|(rid, command)| {
                    let r = team.get_mut(&rid).unwrap(); // TODO: check we have the robot to send the order to
                    r.apply_vel(command.vel);
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
    let ball = Ball::new(Point2::new(0.6, 0.)); // right in front of the goal
    let mut team: HashMap<RobotId, Robot> = HashMap::new();
    team.insert(0, Robot::new(0, Point2::zero()));
    team.insert(1, Robot::new(1, Point2::zero()));
    team.insert(2, Robot::new(2, Point2::zero()));

    let control_loop_thread = launch_control_thread(team.clone(), ball.clone());

    do_square(team.get(&0).unwrap()).await;
    // we simulate a penalty after 2s
    let _ = tokio::time::timeout(
        Duration::from_millis(500),
        three_attackers_attack(
            team.get(&1).unwrap(),
            team.get(&0).unwrap(),
            team.get(&2).unwrap(),
        ),
    )
    .await;

    // now we spin the ball and make the robot try to go get it to showcase the Trackable trait
    make_ball_spin(ball.clone(), Some(Duration::from_secs(5)));
    go_get_ball(team.get(&0).unwrap(), &ball).await;

    control_loop_thread.abort();
}
