#![feature(future_join)]

pub mod actions;
pub mod robot;
pub mod vec2;

use actions::*;
use robot::{Robot, RobotId};
use std::{collections::HashMap, time::Duration};
use tokio::task::JoinHandle;
use vec2::Vec2f32;

const CONTROL_PERIOD: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct RobotCommand {
    vel: Vec2f32,
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

fn launch_control_thread(mut team: HashMap<RobotId, Robot>) -> JoinHandle<()> {
    let mut interval = tokio::time::interval(CONTROL_PERIOD);
    tokio::spawn(async move {
        // Robot control loop, 1Hz
        loop {
            interval.tick().await; // first tick ticks immediately that's why it's at the beginning

            for r in team.values() {
                println!("id: {} | pos: {:?}", r.get_id(), r.get_pos());
            }

            println!("[DEBUG] sending commands!");
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

/// Simulation of a real control loop
#[tokio::main]
async fn main() {
    let mut team: HashMap<RobotId, Robot> = HashMap::new();
    team.insert(0, Robot::new(0, Vec2f32::zero()));
    team.insert(1, Robot::new(1, Vec2f32::zero()));
    team.insert(2, Robot::new(2, Vec2f32::zero()));

    let control_loop_thread = launch_control_thread(team.clone());

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

    control_loop_thread.abort();
}
