use std::ops::{Div, Sub};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Vec2f32 {
    pub x: f32,
    pub y: f32,
}

impl Vec2f32 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0., 0.)
    }

    pub fn norm(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl Sub for Vec2f32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<f32> for Vec2f32 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
