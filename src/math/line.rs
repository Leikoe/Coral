use super::{Point2, Vec2};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Line {
    start: Point2,
    end: Point2,
}

impl Line {
    pub fn new(start: Point2, end: Point2) -> Self {
        Self { start, end }
    }

    pub fn closest_point_to(&self, point: Point2) -> Point2 {
        let line_direction = self.end - self.start;
        let point_direction = point - self.start;

        let line_length_squared = line_direction.norm().powi(2);
        if line_length_squared == 0.0 {
            // The line segment has zero length, return the start point.
            return self.start;
        }
        let t = point_direction.dot(line_direction) / line_length_squared;

        // The point is closest to a point on the segment.
        self.start + line_direction * t
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
