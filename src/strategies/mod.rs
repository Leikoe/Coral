use crate::{Robot, RobotId};
use std::collections::HashMap;

pub mod square;
pub mod three_attackers;

pub trait Strategy {
    async fn run(&self, team: HashMap<RobotId, Robot>);
}
