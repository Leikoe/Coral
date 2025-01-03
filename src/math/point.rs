use super::{Reactive, Vec2};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, DivAssign, MulAssign, Sub, SubAssign};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn to(&self, rhs: Point2) -> Vec2 {
        rhs - *self
    }

    pub fn from_vec(point: &[f64]) -> Self {
        Self::new(point[0], point[1])
    }

    pub fn to_vec(self) -> Vec<f64> {
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

impl Reactive<Point2> for Point2 {
    fn get_reactive(&self) -> Point2 {
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

impl MulAssign<f64> for Point2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f64> for Point2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl AsRef<Point2> for Point2 {
    fn as_ref(&self) -> &Point2 {
        self
    }
}
