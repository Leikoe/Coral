use crate::{trackable::Trackable, Point2};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Ball {
    pos: Arc<Mutex<Point2>>,
}

impl Ball {
    pub fn new(pos: Point2) -> Self {
        Self {
            pos: Arc::new(Mutex::new(pos)),
        }
    }

    // this is used for testing
    pub fn set_pos(&self, pos: Point2) {
        let mut self_pos = self.pos.lock().unwrap();
        *self_pos = pos;
    }
}

impl Trackable for Ball {
    fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }
}
