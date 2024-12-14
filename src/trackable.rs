use crate::{Point2, Vec2};

pub trait Trackable {
    fn get_pos(&self) -> Point2;
    fn to<T: Trackable>(&self, rhs: &T) -> Vec2 {
        self.get_pos().to(rhs.get_pos())
    }
    fn distance<T: Trackable>(&self, rhs: &T) -> f32 {
        self.get_pos().distance_to(rhs.get_pos())
    }
}

impl<T: Fn() -> R, R: Trackable> Trackable for T {
    fn get_pos(&self) -> Point2 {
        self().get_pos()
    }
}
