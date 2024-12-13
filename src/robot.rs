use tokio::time::Interval;

use crate::{vec2::Vec2f32, RobotCommand, CONTROL_PERIOD};
use std::sync::{Arc, Mutex};

pub type RobotId = u8;
const IS_CLOSE_EPSILON: f32 = 0.01;

#[derive(Clone)]
pub struct Robot {
    id: RobotId,
    pos: Arc<Mutex<Vec2f32>>,
    pub next_command: Arc<Mutex<Option<RobotCommand>>>,
}

impl Robot {
    pub fn new(id: RobotId, pos: Vec2f32) -> Self {
        Self {
            id,
            pos: Arc::new(Mutex::new(pos)),
            next_command: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
    }

    pub fn get_pos(&self) -> Vec2f32 {
        self.pos.lock().unwrap().clone()
    }

    pub fn apply_vel(&mut self, vel: Vec2f32) {
        let mut pos = self.pos.lock().unwrap();
        pos.x += vel.x;
        pos.y += vel.y;
    }

    pub async fn goto(&self, pos: Vec2f32) {
        let mut cur_pos = self.get_pos();
        let mut to_pos = pos - cur_pos;

        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while to_pos.norm() > IS_CLOSE_EPSILON {
            {
                let mut next_command = self.next_command.lock().unwrap();
                next_command.replace(RobotCommand {
                    vel: Vec2f32::new(to_pos.x / 10., to_pos.y / 10.),
                    kick: false,
                });
            }
            interval.tick().await;
            cur_pos = self.get_pos();
            to_pos = pos - cur_pos;
        }
    }

    // what can you wait for a robot to do ?
    // - goto(pos)
    // - kick()
    // ??
}
