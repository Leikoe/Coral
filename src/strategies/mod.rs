use crate::{Robot, RobotId};
use std::collections::HashMap;

pub mod square;

pub trait Strategy {
    async fn run(&self, team: HashMap<RobotId, Robot>);
}
