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

#[derive(Clone)]
pub struct World {
    pub team_color: TeamColor,
    pub field: Field, // already has light cloning because internal arcs
    pub ball: Ball,   // already has light cloning because internal arcs
    pub team: Arc<Mutex<HashMap<RobotId, AllyRobot>>>,
    pub ennemies: Arc<Mutex<HashMap<RobotId, EnnemyRobot>>>,
}

impl World {
    pub fn default_with_team_color(team_color: TeamColor) -> Self {
        Self {
            team_color,
            field: Field::default(),
            ball: Ball::default(),
            team: Default::default(),
            ennemies: Default::default(),
        }
    }

    pub fn get_ennemy_goal_bounding_box(&self) -> Rect {
        match self.team_color {
            TeamColor::Blue => self.field.get_yellow_goal_bounding_box(),
            TeamColor::Yellow => self.field.get_blue_goal_bounding_box(),
        }
    }
}

#[derive(Clone)]
pub struct Field {
    /// field's length in meters
    field_length: Arc<Mutex<f64>>,
    /// field's width in meters
    field_width: Arc<Mutex<f64>>,
    // /// Goal width (distance between inner edges of goal posts) in m
    goal_width: Arc<Mutex<f64>>,
    /// Goal depth (distance from outer goal line edge to inner goal back) in m
    goal_depth: Arc<Mutex<f64>>,
}

impl Default for Field {
    /// defaults to div B size
    fn default() -> Self {
        Field {
            field_length: Arc::new(Mutex::new(9.)),
            field_width: Arc::new(Mutex::new(6.)),
            goal_width: Arc::new(Mutex::new(1.)),
            goal_depth: Arc::new(Mutex::new(0.18)),
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

    pub fn get_goal_depth(&self) -> f64 {
        *self.goal_depth.lock().unwrap()
    }

    pub fn get_goal_width(&self) -> f64 {
        *self.goal_width.lock().unwrap()
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

    pub fn get_yellow_goal_bounding_box(&self) -> Rect {
        let x_outer_line = self.get_field_length() / 2.;
        Rect::new(
            Point2::new(x_outer_line as f32, (self.get_goal_width() / 2.) as f32),
            Point2::new(
                (x_outer_line + self.get_goal_depth()) as f32,
                (-self.get_goal_width() / 2.) as f32,
            ),
        )
    }

    pub fn get_blue_goal_bounding_box(&self) -> Rect {
        let x_outer_line = -self.get_field_length() / 2.;
        Rect::new(
            Point2::new(
                (x_outer_line - self.get_goal_depth()) as f32,
                (self.get_goal_width() / 2.) as f32,
            ),
            Point2::new(x_outer_line as f32, (-self.get_goal_width() / 2.) as f32),
        )
    }
}
