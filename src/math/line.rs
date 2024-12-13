use std::ops::{Add, Sub};

use super::{Point2, Vec2};

#[derive(Copy, Clone, Debug)]
struct Line {
    start: Point2,
    end: Point2,
}

impl Line {
    pub fn new(start: Point2, end: Point2) -> Self {
        Self { start, end }
    }
}

impl Add<Vec2> for Line {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self {
            start: self.start + rhs,
            end: self.end + rhs,
        }
    }
}

impl Sub<Vec2> for Line {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self {
            start: self.start - rhs,
            end: self.end - rhs,
        }
    }
}
