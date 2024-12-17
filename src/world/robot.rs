use crate::{
    math::{angle_difference, Point2, Rect, Vec2},
    world::Trackable,
    world::World,
    CONTROL_PERIOD,
};
use std::sync::{Arc, Mutex};

pub type RobotId = u8;
const IS_CLOSE_EPSILON: f32 = 0.05;
const RRT_MAX_TRIES: usize = 10_000;

const GOTO_SPEED: f32 = 1.5;
const GOTO_ANGULAR_SPEED: f32 = 1.5;

#[derive(Clone)]
pub struct Robot {
    id: RobotId,
    pos: Arc<Mutex<Point2>>,
    orientation: Arc<Mutex<f32>>,
    // control stuff
    target_vel: Arc<Mutex<Vec2>>,
    target_angular_vel: Arc<Mutex<f32>>,
    should_dribble: Arc<Mutex<bool>>,
    should_kick: Arc<Mutex<bool>>,
}

impl Trackable for Robot {
    fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }
}

impl Robot {
    pub fn new(id: RobotId, pos: Point2, orientation: f32) -> Self {
        Self {
            id,
            pos: Arc::new(Mutex::new(pos)),
            orientation: Arc::new(Mutex::new(orientation)),
            target_vel: Arc::new(Mutex::new(Vec2::zero())),
            target_angular_vel: Arc::new(Mutex::new(0.)),
            should_dribble: Arc::new(Mutex::new(false)),
            should_kick: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
    }

    pub fn kick(&self) {
        let mut should_kick = self.should_kick.lock().unwrap();
        *should_kick = true;
    }

    // return the should_kick state & resets it back to false (similar to Option::take)
    pub fn take_should_kick(&self) -> bool {
        let mut should_kick = self.should_kick.lock().unwrap();
        let ret = *should_kick;
        *should_kick = false;
        ret
    }

    pub fn enable_dribbler(&self) {
        let mut is_dribbling = self.should_dribble.lock().unwrap();
        *is_dribbling = true;
    }

    pub fn disable_dribbler(&self) {
        let mut is_dribbling = self.should_dribble.lock().unwrap();
        *is_dribbling = false;
    }

    pub fn get_orientation(&self) -> f32 {
        *self.orientation.lock().unwrap()
    }

    pub fn should_dribble(&self) -> bool {
        self.should_dribble.lock().unwrap().clone()
    }

    pub fn apply_vel(&mut self, vel: Vec2) {
        let mut pos = self.pos.lock().unwrap();
        *pos += vel;
    }

    pub fn apply_angular_vel(&mut self, angular_vel: f32) {
        let mut orientation = self.orientation.lock().unwrap();
        *orientation += angular_vel;
    }

    pub fn get_target_vel(&self) -> Vec2 {
        *self.target_vel.lock().unwrap()
    }

    pub fn get_target_angular_vel(&self) -> f32 {
        *self.target_angular_vel.lock().unwrap()
    }

    pub fn set_target_vel(&self, target_vel: Vec2) {
        let mut self_target_vel = self.target_vel.lock().unwrap();
        *self_target_vel = target_vel;
    }

    pub fn set_target_angular_vel(&self, target_angular_vel: f32) {
        let mut self_target_angular_vel = self.target_angular_vel.lock().unwrap();
        *self_target_angular_vel = target_angular_vel;
    }

    // destination is in world space
    pub async fn goto<T: Trackable>(&self, destination: &T, angle: Option<f32>) {
        let mut cur_pos = self.get_pos();
        let mut to_pos = destination.get_pos() - cur_pos;

        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while to_pos.norm() > IS_CLOSE_EPSILON {
            let self_orientation = self.get_orientation();
            let inverse_orientation = -self_orientation;
            let robot_to_rest_robot_pov = Point2::new(
                to_pos.x * inverse_orientation.cos() - to_pos.y * inverse_orientation.sin(),
                to_pos.y * inverse_orientation.cos() + to_pos.x * inverse_orientation.sin(),
            );

            let angle_diff = angle
                .map(|x| angle_difference(x as f64, self_orientation as f64) as f32)
                .unwrap_or_default();
            self.set_target_vel(Vec2::new(
                robot_to_rest_robot_pov.x * GOTO_SPEED,
                robot_to_rest_robot_pov.y * GOTO_SPEED,
            ));
            self.set_target_angular_vel(angle_diff * GOTO_ANGULAR_SPEED);

            // next iter starts here
            interval.tick().await;
            cur_pos = self.get_pos(); // compute diff
            to_pos = destination.get_pos() - cur_pos;
        }
    }

    pub fn debug_tp<T: Trackable>(&self, destination: &T, angle: Option<f32>) {
        let mut pos = self.pos.lock().unwrap();
        *pos = destination.get_pos();

        let angle = angle.unwrap_or(self.get_orientation());
        let mut orientation = self.orientation.lock().unwrap();
        *orientation = angle;
    }

    pub fn set_pos(&mut self, pos: Point2) {
        let mut _pos = self.pos.lock().unwrap();
        *_pos = pos;
    }

    pub fn set_orientation(&mut self, orientation: f32) {
        let mut _orientation = self.orientation.lock().unwrap();
        *_orientation = orientation;
    }

    pub async fn goto_rrt<T: Trackable>(
        &self,
        world: &Arc<Mutex<World>>,
        destination: &T,
        angle: Option<f32>,
    ) -> Result<Vec<Point2>, String> {
        let is_free = |p: &[f32]| {
            let p = Point2::from_vec(p);
            !world
                .lock()
                .unwrap()
                .team
                .values()
                .filter(|r| r.get_id() != self.get_id()) // can't collide with myself
                .any(|r| p.distance_to(r) < 0.3) // a robot is 10cm radius => 0.3 leaves 10cm between robots
        };

        let mut followed_path = vec![self.get_pos()];
        while self.get_pos().distance_to(&destination.get_pos()) > IS_CLOSE_EPSILON {
            let start = self.get_pos();
            let goal = destination.get_pos();
            // let rect = Rect::new(Point2::new(start.x, 1.75), Point2::new(goal.x, -1.75));
            let rect = world.lock().unwrap().field;

            let start_time = tokio::time::Instant::now();
            let mut path = rrt::dual_rrt_connect(
                &[start.x, start.y],
                &[goal.x, goal.y],
                is_free,
                || rect.sample_inside().to_vec(),
                0.1,
                RRT_MAX_TRIES,
            )?;
            rrt::smooth_path(&mut path, is_free, 0.05, 100);
            let next_point = path
                .into_iter()
                .nth(1)
                .ok_or(format!("Couldn't find a path to {:?}", goal))?;
            println!(
                "[DEBUG - robot {} - goto_rrt] took {}ms to compute path",
                self.get_id(),
                start_time.elapsed().as_millis()
            );

            self.goto(&Point2::from_vec(&next_point), angle).await;
            followed_path.push(self.get_pos());
        }
        Ok(followed_path)
    }

    // what can you wait for a robot to do ?
    // - goto(pos)
    // - kick()
    // ??
}
