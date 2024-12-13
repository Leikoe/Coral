use super::Strategy;
use crate::{Robot, RobotId, Vec2f32};
use std::{collections::HashMap, future::join};

pub struct ThreeAttackersStrategy;

impl Strategy for ThreeAttackersStrategy {
    async fn run(&self, team: HashMap<RobotId, Robot>) {
        let left_winger = team.get(&1).unwrap();
        let right_winger = team.get(&2).unwrap();
        let fronter = team.get(&0).unwrap();
        join!(
            left_winger.goto(Vec2f32::new(0.5, 0.5)),
            right_winger.goto(Vec2f32::new(0.5, -0.5)),
            fronter.goto(Vec2f32::new(0.5, 0.)),
        )
        .await;
    }
}
