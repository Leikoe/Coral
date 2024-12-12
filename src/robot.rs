use crate::{vec2::Vec2f32, RobotCommand};

pub type RobotId = u8;

pub struct Robot {
    id: RobotId,
    pos: Vec2f32,
    pub next_command: Option<RobotCommand>,
}

impl Robot {
    pub fn new(id: RobotId, pos: Vec2f32) -> Self {
        Self {
            id,
            pos,
            next_command: None,
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
    }

    pub fn get_pos(&self) -> Vec2f32 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Vec2f32) {
        self.pos = pos;
    }

    pub fn apply_vel(&mut self, vel: Vec2f32) {
        self.pos.x += vel.x;
        self.pos.y += vel.y;
    }
}
