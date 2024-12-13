use super::vec::Vec2;
use crate::trackable::Trackable;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0., y: 0. }
    }

    pub fn to(&self, rhs: Point2) -> Vec2 {
        rhs - *self
    }

    pub fn distance_to(&self, rhs: Point2) -> f32 {
        (rhs - *self).norm()
    }
}

impl Sub for Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add<Vec2> for Point2 {
    type Output = Point2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vec2> for Point2 {
    type Output = Point2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Point2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Trackable for Point2 {
    fn get_pos(&self) -> Point2 {
        *self
    }
}
