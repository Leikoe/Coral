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
}

impl Trackable for Ball {
    fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }
}
