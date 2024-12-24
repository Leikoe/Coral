use super::{Point2, Vec2};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Line {
    start: Point2,
    end: Point2,
}

pub struct LinesParallelError;

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

    pub fn intersection_lines(&self, line: &Line) -> Result<Point2, LinesParallelError> {
        let line_direction = self.end - self.start;
        let other_direction = line.end - line.start;

        // Calculate cross product to check if lines are parallel
        let cross_product =
            line_direction.x * other_direction.y - line_direction.y * other_direction.x;
        if cross_product.abs() < f64::EPSILON {
            return Err(LinesParallelError);
        }

        // Calculate intersection point using determinant method
        let x1 = self.start.x;
        let y1 = self.start.y;
        let x2 = self.end.x;
        let y2 = self.end.y;
        let x3 = line.start.x;
        let y3 = line.start.y;
        let x4 = line.end.x;
        let y4 = line.end.y;

        let denominator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denominator;

        let intersection_x = x1 + t * (x2 - x1);
        let intersection_y = y1 + t * (y2 - y1);

        Ok(Point2::new(intersection_x, intersection_y))
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
