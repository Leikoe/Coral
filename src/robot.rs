use crate::{
    angle_difference, trackable::Trackable, Point2, RobotCommand, Vec2, World, CONTROL_PERIOD,
};
use rand::distributions::{Distribution, Uniform};
use std::sync::{Arc, Mutex};

pub type RobotId = u8;
const IS_CLOSE_EPSILON: f32 = 0.01;
const RRT_MAX_TRIES: usize = 10_000;

#[derive(Clone)]
pub struct Robot {
    id: RobotId,
    pos: Arc<Mutex<Point2>>,
    orientation: Arc<Mutex<f32>>,
    // control stuff
    target_vel: Arc<Mutex<Vec2>>,
    target_angular_vel: Arc<Mutex<f32>>,
    should_dribble: Arc<Mutex<bool>>,
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
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
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

    pub async fn goto<T: Trackable>(&self, destination: &T, angle: Option<f32>) {
        let mut cur_pos = self.get_pos();
        let mut to_pos = destination.get_pos() - cur_pos;

        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while to_pos.norm() > IS_CLOSE_EPSILON {
            let angle_diff = angle
                .map(|x| angle_difference(x as f64, self.get_orientation() as f64) as f32)
                .unwrap_or_default();
            self.set_target_vel(Vec2::new(to_pos.x / 10., to_pos.y / 10.));
            self.set_target_angular_vel(angle_diff / 10.);

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

    pub async fn goto_rrt<T: Trackable>(
        &self,
        world: &World,
        destination: &T,
        angle: Option<f32>,
    ) -> Result<Vec<Point2>, String> {
        let mut followed_path = Vec::new();
        while self.get_pos().distance_to(&destination.get_pos()) > IS_CLOSE_EPSILON {
            let start = self.get_pos();
            let goal = destination.get_pos();
            let start_time = tokio::time::Instant::now();
            let result = rrt::dual_rrt_connect(
                &[start.x, start.y],
                &[goal.x, goal.y],
                |p: &[f32]| {
                    let p = Point2::new(p[0], p[1]);
                    !world.team.values().any(|r| p.distance_to(r) < 0.5)
                },
                || {
                    let between = Uniform::new(-2.0, 2.0);
                    let mut rng = rand::thread_rng();
                    vec![between.sample(&mut rng), between.sample(&mut rng)]
                },
                0.5,
                RRT_MAX_TRIES,
            )?
            .into_iter()
            .nth(1)
            .ok_or(format!("Couldn't find a path to {:?}", goal))?;
            println!(
                "[DEBUG] took {}s to compute path",
                start_time.elapsed().as_secs_f64()
            );

            self.goto(&Point2::new(result[0], result[1]), angle).await;
            followed_path.push(self.get_pos());
        }
        Ok(followed_path)
    }

    pub fn make_command(&self) -> RobotCommand {
        RobotCommand {
            vel: self.get_target_vel(),
            angular_vel: self.get_target_angular_vel(),
            kick: false,
            dribble: self.should_dribble(),
        }
    }

    // what can you wait for a robot to do ?
    // - goto(pos)
    // - kick()
    // ??
}
