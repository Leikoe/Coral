#[allow(async_fn_in_trait)]
pub mod actions;
pub mod controllers;
pub mod league_protocols;
pub mod math;
pub mod net;
pub mod vision;
pub mod world;

use std::time::Duration;

pub const CONTROL_PERIOD: Duration = Duration::from_millis(100);
