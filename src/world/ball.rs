use crate::{
    math::{Point2, Reactive, Vec2},
    IgnoreMutexErr,
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Ball {
    pos: Arc<Mutex<Point2>>,
    vel: Arc<Mutex<Vec2>>,
    last_update: Arc<Mutex<Option<f64>>>,
}

impl Default for Ball {
    fn default() -> Self {
        Ball::new(Default::default(), Default::default())
    }
}

impl Ball {
    pub fn new(pos: Point2, vel: Vec2) -> Self {
        Self {
            pos: Arc::new(Mutex::new(pos)),
            vel: Arc::new(Mutex::new(vel)),
            last_update: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap_ignore_poison()
    }

    pub fn set_pos(&self, pos: Point2) {
        let mut self_pos = self.pos.lock().unwrap_ignore_poison();
        *self_pos = pos;
    }

    pub fn get_vel(&self) -> Vec2 {
        *self.vel.lock().unwrap_ignore_poison()
    }

    pub fn get_last_update(&self) -> Option<f64> {
        *self.last_update.lock().unwrap_ignore_poison()
    }

    pub fn set_last_update(&self, last_update: f64) {
        *self.last_update.lock().unwrap_ignore_poison() = Some(last_update);
    }

    pub fn set_vel(&self, vel: Vec2) {
        *self.vel.lock().unwrap_ignore_poison() = vel;
    }
}

impl Reactive<Point2> for Ball {
    fn get_reactive(&self) -> Point2 {
        self.get_pos()
    }
}

impl AsRef<Ball> for Ball {
    fn as_ref(&self) -> &Ball {
        self
    }
}
