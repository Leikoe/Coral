use super::Strategy;
use crate::{Robot, RobotId, Vec2f32};
use std::collections::HashMap;

pub struct SquareStrategy;

impl Strategy for SquareStrategy {
    async fn run(&self, team: HashMap<RobotId, Robot>) {
        let robot = team.get(&0).unwrap();
        robot.goto(Vec2f32::new(0., 1.)).await;
        robot.goto(Vec2f32::new(1., 1.)).await;
        robot.goto(Vec2f32::new(1., 0.)).await;
        robot.goto(Vec2f32::new(0., 0.)).await;
        println!("reached dest!");
    }
}
