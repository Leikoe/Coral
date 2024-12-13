use crate::Point2;

pub trait Trackable {
    fn get_pos(&self) -> Point2;
}
