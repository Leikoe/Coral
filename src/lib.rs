pub mod actions;
pub mod league_protocols;
pub mod math;
pub mod net;
pub mod world;

use std::time::Duration;

pub const CONTROL_PERIOD: Duration = Duration::from_millis(10);
