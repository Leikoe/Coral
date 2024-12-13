use crate::{trackable::Trackable, Point2, RobotCommand, Vec2, CONTROL_PERIOD};
use std::sync::{Arc, Mutex};

pub type RobotId = u8;
const IS_CLOSE_EPSILON: f32 = 0.01;

#[derive(Clone)]
pub struct Robot {
    id: RobotId,
    pos: Arc<Mutex<Point2>>,
    is_dribbling: Arc<Mutex<bool>>,
    pub next_command: Arc<Mutex<Option<RobotCommand>>>,
}

impl Trackable for Robot {
    fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }
}

impl Robot {
    pub fn new(id: RobotId, pos: Point2) -> Self {
        Self {
            id,
            pos: Arc::new(Mutex::new(pos)),
            is_dribbling: Arc::new(Mutex::new(false)),
            next_command: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
    }

    pub fn enable_dribbler(&self) {
        let mut is_dribbling = self.is_dribbling.lock().unwrap();
        *is_dribbling = true;
    }

    pub fn disable_dribbler(&self) {
        let mut is_dribbling = self.is_dribbling.lock().unwrap();
        *is_dribbling = false;
    }

    pub fn is_dribbling(&self) -> bool {
        self.is_dribbling.lock().unwrap().clone()
    }

    pub fn apply_vel(&mut self, vel: Vec2) {
        let mut pos = self.pos.lock().unwrap();
        pos.x += vel.x;
        pos.y += vel.y;
    }

    pub async fn goto<T: Trackable>(&self, destination: T) {
        let mut cur_pos = self.get_pos();
        let mut to_pos = destination.get_pos() - cur_pos;

        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while to_pos.norm() > IS_CLOSE_EPSILON {
            {
                let mut next_command = self.next_command.lock().unwrap();
                next_command.replace(RobotCommand {
                    vel: Vec2::new(to_pos.x / 10., to_pos.y / 10.),
                    kick: false,
                    dribble: self.is_dribbling(),
                });
            }
            interval.tick().await;
            cur_pos = self.get_pos();
            to_pos = destination.get_pos() - cur_pos;
        }
    }

    // what can you wait for a robot to do ?
    // - goto(pos)
    // - kick()
    // ??
}
