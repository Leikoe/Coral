pub mod line;
pub mod rect;

pub use line::*;
use na;
pub use rect::*;

use std::f64::consts::{PI, TAU};

pub type Point2 = na::Point2<f64>;
pub type Vec2 = na::Vector2<f64>;

pub trait Angle {
    fn _angle(&self) -> f64;
}

impl Angle for Vec2 {
    fn _angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
}

impl<F: Fn() -> f64> Angle for F {
    fn _angle(&self) -> f64 {
        self()
    }
}

pub fn angle_difference(alpha1: f64, alpha2: f64) -> f64 {
    let diff = alpha1 - alpha2;
    match diff {
        d if d > PI => d - TAU,
        d if d < -PI => d + TAU,
        d => d,
    }
}
