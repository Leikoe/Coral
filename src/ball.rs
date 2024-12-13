use crate::{math::Vec2, trackable::Trackable};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Ball {
    pos: Arc<Mutex<Vec2>>,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos: Arc::new(Mutex::new(pos)),
        }
    }
}

impl Trackable for Ball {
    fn get_pos(&self) -> Vec2 {
        self.pos.lock().unwrap().clone()
    }
}
