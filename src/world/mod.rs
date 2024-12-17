mod ball;
mod robot;
mod trackable;

// EXPORTS
pub use ball::Ball;
pub use robot::{AvoidanceMode, Robot, RobotId};
pub use trackable::*;

use crate::math::Rect;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TeamColor {
    Blue,
    Yellow,
}

#[derive(Clone)]
pub struct World {
    pub field: Rect,
    pub ball: Ball,
    pub team: HashMap<RobotId, Robot>,
}
