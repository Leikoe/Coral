mod ball;
mod robot;

// EXPORTS
pub use ball::Ball;
pub use robot::{AllyRobot, AvoidanceMode, EnnemyRobot, RobotId};

use crate::math::Rect;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TeamColor {
    Blue,
    Yellow,
}

#[derive(Clone)]
pub struct World {
    pub field: Arc<Mutex<Rect>>,
    pub ball: Ball, // already has light cloning because internal arcs
    pub team: Arc<Mutex<HashMap<RobotId, AllyRobot>>>,
    pub ennemies: Arc<Mutex<HashMap<RobotId, EnnemyRobot>>>,
}
