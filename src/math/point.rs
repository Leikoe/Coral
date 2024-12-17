use super::Vec2;
use crate::world::Trackable;
use std::ops::{Add, AddAssign, DivAssign, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0., 0.)
    }

    pub fn to(&self, rhs: Point2) -> Vec2 {
        rhs - *self
    }

    pub fn from_vec(point: &[f32]) -> Self {
        Self::new(point[0], point[1])
    }

    pub fn to_vec(self) -> Vec<f32> {
        vec![self.x, self.y]
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

impl Add<&Vec2> for Point2 {
    type Output = Point2;

    fn add(self, rhs: &Vec2) -> Self::Output {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<&Vec2> for Point2 {
    type Output = Point2;

    fn sub(self, rhs: &Vec2) -> Self::Output {
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

impl AddAssign<Vec2> for Point2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl SubAssign<Vec2> for Point2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl MulAssign<f32> for Point2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f32> for Point2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl AsRef<Point2> for Point2 {
    fn as_ref(&self) -> &Point2 {
        self
    }
}
