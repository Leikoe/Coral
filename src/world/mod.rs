mod ball;
mod robot;

// EXPORTS
pub use ball::Ball;
pub use robot::{AllyRobot, AvoidanceMode, EnnemyRobot, RobotId};

use crate::{
    league_protocols::vision_packet::SslGeometryFieldSize,
    math::{Point2, Rect},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TeamColor {
    Blue,
    Yellow,
}

#[derive(Clone, Default)]
pub struct World {
    pub field: Field, // already has light cloning because internal arcs
    pub ball: Ball,   // already has light cloning because internal arcs
    pub team: Arc<Mutex<HashMap<RobotId, AllyRobot>>>,
    pub ennemies: Arc<Mutex<HashMap<RobotId, EnnemyRobot>>>,
}

#[derive(Clone)]
pub struct Field {
    /// field's length in meters
    field_length: Arc<Mutex<f64>>,
    /// field's width in meters
    field_width: Arc<Mutex<f64>>,
}

impl Default for Field {
    /// defaults to div B size
    fn default() -> Self {
        Field {
            field_length: Arc::new(Mutex::new(9.)),
            field_width: Arc::new(Mutex::new(6.)),
        }
    }
}

impl Field {
    pub fn update_from_packet(&mut self, packet: SslGeometryFieldSize) {
        *self.field_length.lock().unwrap() = packet.field_length as f64 / 1000.;
        *self.field_width.lock().unwrap() = packet.field_width as f64 / 1000.;
    }

    pub fn get_field_length(&self) -> f64 {
        *self.field_length.lock().unwrap()
    }

    pub fn get_field_width(&self) -> f64 {
        *self.field_width.lock().unwrap()
    }

    pub fn get_bounding_box(&self) -> Rect {
        Rect::new(
            Point2::new(
                (self.get_field_width() / 2.) as f32,
                (-self.get_field_length() / 2.) as f32,
            ),
            Point2::new(
                (-self.get_field_width() / 2.0) as f32,
                (self.get_field_length() / 2.0) as f32,
            ),
        )
    }
}
