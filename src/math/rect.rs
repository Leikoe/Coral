use super::point::Point2;
use rand::distributions::{Distribution, Uniform};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    top_left: Point2,
    bottom_right: Point2,
}

impl Rect {
    pub fn new(p1: Point2, p2: Point2) -> Self {
        let min_x = p1.x.min(p2.x);
        let max_x = p1.x.max(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_y = p1.y.max(p2.y);

        Self {
            top_left: Point2::new(min_x, max_y),
            bottom_right: Point2::new(max_x, min_y),
        }
    }

    pub fn sample_inside(&self) -> Point2 {
        let mut rng = rand::thread_rng();
        Point2::new(
            Uniform::new(self.top_left.x, self.bottom_right.x).sample(&mut rng),
            Uniform::new(self.bottom_right.y, self.top_left.y).sample(&mut rng),
        )
    }
}
