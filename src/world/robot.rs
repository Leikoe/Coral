use tokio::time::sleep;
use trajectory::{CubicSpline, Trajectory};

use crate::{
    league_protocols::vision_packet::SslDetectionRobot,
    math::{angle_difference, Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext, Rect, Vec2},
    world::World,
    CONTROL_PERIOD, DETECTION_SCALING_FACTOR,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use super::Ball;

pub type RobotId = u8;
const IS_CLOSE_EPSILON: f32 = 0.05;
const RRT_MAX_TRIES: usize = 1_000;

const GOTO_SPEED: f32 = 1.5;
const GOTO_ANGULAR_SPEED: f32 = 1.5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AvoidanceMode {
    /// can collide with everything
    None,
    /// can collide with ball, can't collide with robots
    AvoidRobots,
    /// can't collide
    AvoidRobotsAndBall,
}

// TODO: RobotData, Robot, AllyData and EnnemyData should be private
pub trait RobotData: Clone + Default {}

#[derive(Clone, Default)]
pub struct Robot<D: RobotData> {
    id: RobotId,
    pos: Arc<Mutex<Point2>>,
    orientation: Arc<Mutex<f32>>,
    has_ball: Arc<Mutex<bool>>,
    internal_data: D,
}

#[derive(Clone, Default)]
pub struct AllyData {
    target_vel: Arc<Mutex<Vec2>>,
    target_angular_vel: Arc<Mutex<f32>>,
    should_dribble: Arc<Mutex<bool>>,
    should_kick: Arc<Mutex<bool>>,
}

impl RobotData for AllyData {}

#[derive(Clone, Default)]
pub struct EnnemyData;

impl RobotData for EnnemyData {}

pub type AllyRobot = Robot<AllyData>;
pub type EnnemyRobot = Robot<EnnemyData>;

impl<D: RobotData> Reactive<Point2> for Robot<D> {
    fn get_reactive(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }
}

impl<D: RobotData> Robot<D> {
    pub fn new(id: RobotId, pos: Point2, orientation: f32) -> Self {
        Self {
            id,
            pos: Arc::new(Mutex::new(pos)),
            orientation: Arc::new(Mutex::new(orientation)),
            has_ball: Arc::new(Mutex::new(false)),
            internal_data: Default::default(),
        }
    }

    pub fn default_with_id(id: RobotId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
    }

    pub fn has_ball(&self) -> bool {
        *self.has_ball.lock().unwrap()
    }

    pub fn set_has_ball(&mut self, has_ball: bool) {
        let mut _has_ball = self.has_ball.lock().unwrap();
        *_has_ball = has_ball;
    }

    pub fn get_orientation(&self) -> f32 {
        *self.orientation.lock().unwrap()
    }

    pub fn debug_tp(&self, destination: Point2, angle: Option<f32>) {
        let mut pos = self.pos.lock().unwrap();
        *pos = destination;

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

    pub fn update_from_packet(&mut self, detection: SslDetectionRobot, ball: &Ball) {
        let detected_pos = Point2::new(
            detection.x / DETECTION_SCALING_FACTOR,
            detection.y / DETECTION_SCALING_FACTOR,
        );
        let detected_orientation = detection.orientation();
        self.set_orientation(detected_orientation);
        self.set_pos(detected_pos);
        let has_ball = {
            let r_to_ball = self.to(ball);
            let is_facing_ball =
                angle_difference(r_to_ball.angle() as f64, self.get_orientation() as f64).abs()
                    < 20.;
            is_facing_ball && (r_to_ball.norm() < 0.15) // TODO: stop the magic
        };
        self.set_has_ball(has_ball);
    }

    fn collides_with_robot(&self, other_pos: Point2) -> bool {
        self.distance_to(&other_pos) < 0.3 // a robot is 10cm radius => 0.3 leaves 10cm between robots
    }

    fn pov(&self, pos: Point2) -> Point2 {
        let to_pos = self.to(&pos).get_reactive();
        let self_orientation = self.get_orientation();
        let inverse_orientation = -self_orientation;
        Point2::new(
            to_pos.x * inverse_orientation.cos() - to_pos.y * inverse_orientation.sin(),
            to_pos.y * inverse_orientation.cos() + to_pos.x * inverse_orientation.sin(),
        )
    }
}

impl Robot<AllyData> {
    pub fn kick(&self) {
        let mut should_kick = self.internal_data.should_kick.lock().unwrap();
        *should_kick = true;
    }

    // return the should_kick state & resets it back to false (similar to Option::take)
    pub fn take_should_kick(&self) -> bool {
        let mut should_kick = self.internal_data.should_kick.lock().unwrap();
        let ret = *should_kick;
        *should_kick = false;
        ret
    }

    pub fn enable_dribbler(&self) {
        let mut is_dribbling = self.internal_data.should_dribble.lock().unwrap();
        *is_dribbling = true;
    }

    pub fn disable_dribbler(&self) {
        let mut is_dribbling = self.internal_data.should_dribble.lock().unwrap();
        *is_dribbling = false;
    }

    pub fn should_dribble(&self) -> bool {
        *self.internal_data.should_dribble.lock().unwrap()
    }

    pub fn get_target_vel(&self) -> Vec2 {
        *self.internal_data.target_vel.lock().unwrap()
    }

    pub fn get_target_angular_vel(&self) -> f32 {
        *self.internal_data.target_angular_vel.lock().unwrap()
    }

    pub fn set_target_vel(&self, target_vel: Vec2) {
        let mut self_target_vel = self.internal_data.target_vel.lock().unwrap();
        *self_target_vel = target_vel;
    }

    pub fn set_target_angular_vel(&self, target_angular_vel: f32) {
        let mut self_target_angular_vel = self.internal_data.target_angular_vel.lock().unwrap();
        *self_target_angular_vel = target_angular_vel;
    }

    // destination is in world space
    pub async fn goto<T: Reactive<Point2>>(
        &self,
        destination: &T,
        angle: Option<f32>,
    ) -> Vec<Point2> {
        let mut cur_pos = self.get_reactive();
        let mut followed_path = vec![cur_pos];
        let mut to_pos = destination.get_reactive() - cur_pos;

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
            cur_pos = self.get_reactive(); // compute diff
            to_pos = destination.get_reactive() - cur_pos;
            followed_path.push(cur_pos);
        }
        followed_path
    }

    fn is_free(
        &self,
        pos: Point2,
        world: &Arc<Mutex<World>>,
        avoidance_mode: AvoidanceMode,
    ) -> bool {
        if let AvoidanceMode::None = avoidance_mode {
            return true;
        }

        let is_colliding_with_allies = world
            .lock()
            .unwrap()
            .team
            .values()
            .filter(|r| r.get_id() != self.get_id()) // can't collide with myself
            .any(|r| r.collides_with_robot(pos));
        let is_colliding_with_ennemies = world
            .lock()
            .unwrap()
            .ennemies
            .values()
            .any(|r| r.collides_with_robot(pos));

        let is_colliding_with_a_robot = is_colliding_with_allies || is_colliding_with_ennemies;
        if let AvoidanceMode::AvoidRobots = avoidance_mode {
            return !is_colliding_with_a_robot;
        }

        let is_colliding_with_ball = pos.distance_to(&world.lock().unwrap().ball) < 0.2;
        return !is_colliding_with_a_robot && !dbg!(is_colliding_with_ball);
    }

    pub fn is_a_valid_trajectory(
        &self,
        traj: &CubicSpline<f32>,
        world: &Arc<Mutex<World>>,
        avoidance_mode: AvoidanceMode,
    ) -> bool {
        const TIME_STEP: f32 = 200.; // 200ms as per tiger's tdp
        for i in 0.. {
            let p = match traj.position(i as f32 * TIME_STEP) {
                Some(p) => Point2::from_vec(&p),
                None => {
                    return true;
                }
            };
            if !self.is_free(p, world, avoidance_mode) {
                return false;
            }
        }
        true
    }

    pub async fn goto_rrt<T: Reactive<Point2>>(
        &self,
        world: &Arc<Mutex<World>>,
        destination: &T,
        angle: Option<f32>,
        avoidance_mode: AvoidanceMode,
    ) -> Result<Vec<Point2>, String> {
        // fallback to Robot::goto
        if let AvoidanceMode::None = avoidance_mode {
            return Ok(self.goto(destination, angle).await);
        }

        if !self.is_free(self.get_reactive(), world, avoidance_mode) {
            return Err("we are in a position which isn't free".to_string());
        }

        let is_angle_right = || match angle {
            Some(angle) => angle_difference(angle as f64, self.get_orientation() as f64) < 10.,
            None => true,
        };

        let mut followed_path = vec![self.get_reactive()];
        while self.get_reactive().distance_to(&destination.get_reactive()) > IS_CLOSE_EPSILON
            && is_angle_right()
        {
            let start = Point2::zero();
            let goal = self.pov(destination.get_reactive());

            // let rect = Rect::new(Point2::new(start.x, 1.75), Point2::new(goal.x, -1.75));
            let rect = world.lock().unwrap().field;

            let start_time = tokio::time::Instant::now();
            let mut path = rrt::dual_rrt_connect(
                &[start.x, start.y],
                &[goal.x, goal.y],
                |p| self.is_free(Point2::from_vec(p), world, avoidance_mode),
                || rect.sample_inside().to_vec(),
                0.1,
                RRT_MAX_TRIES,
            )?;
            rrt::smooth_path(
                &mut path,
                |p| self.is_free(Point2::from_vec(p), world, avoidance_mode),
                0.05,
                100,
            );
            let next_point = path
                .into_iter()
                .nth(1)
                .ok_or(format!("Couldn't find a path to {:?}", goal))?;
            println!(
                "[TRACE - robot {} - goto_rrt] took {}ms to compute path",
                self.get_id(),
                start_time.elapsed().as_millis()
            );

            self.goto(&Point2::from_vec(&next_point), angle).await;
            followed_path.push(self.get_reactive());
        }
        Ok(followed_path)
    }

    pub async fn goto_traj<T: Reactive<Point2>>(
        &self,
        world: &Arc<Mutex<World>>,
        destination: &T,
        angle: Option<f32>,
        avoidance_mode: AvoidanceMode,
    ) -> Result<Vec<Point2>, String> {
        // fallback to Robot::goto
        if let AvoidanceMode::None = avoidance_mode {
            return Ok(self.goto(destination, angle).await);
        }

        if !self.is_free(self.get_reactive(), world, avoidance_mode) {
            return Err("we are in a position which isn't free".to_string());
        }

        let is_angle_right = || match angle {
            Some(angle) => angle_difference(angle as f64, self.get_orientation() as f64) < 10.,
            None => true,
        };

        let mut followed_path = vec![self.get_reactive()];
        while self.get_reactive().distance_to(&destination.get_reactive()) > IS_CLOSE_EPSILON
            && is_angle_right()
        {
            let start = Point2::zero();
            let goal = self.pov(destination.get_reactive());
            let field = world.lock().unwrap().field;

            let path = rrt::dual_rrt_connect(
                &[start.x, start.y],
                &[goal.x, goal.y],
                |p| self.is_free(Point2::from_vec(p), world, avoidance_mode),
                || field.sample_inside().to_vec(),
                0.1,
                RRT_MAX_TRIES,
            )?;
            const AVG_SPEED: f32 = 2.; // robot avg speed
            let mut last = path.first().unwrap();
            let mut last_time = 0.;
            let mut times = vec![last_time];
            for p in path.iter().skip(1) {
                let d = Point2::from_vec(last).distance_to(&Point2::from_vec(p));
                let t = d / AVG_SPEED;
                last_time += t;
                times.push(last_time);
                last = p;
            }

            let start = Instant::now();
            let trajectory = CubicSpline::new(times, path).unwrap();
            while self.is_a_valid_trajectory(&trajectory, world, avoidance_mode) {
                let elapsed_s = start.elapsed().as_secs_f32();
                let v = match trajectory.velocity(elapsed_s) {
                    Some(v) => Vec2::from_vec(&v),
                    None => {
                        self.set_target_vel(Vec2::zero());
                        break; // Done
                    }
                };
                let p = Point2::from_vec(&trajectory.position(elapsed_s).unwrap()); // for now assume that we returned above if it was done
                let p_diff = p - self.get_reactive();
                self.set_target_vel(v + p_diff * 0.1);
                sleep(CONTROL_PERIOD).await;
                followed_path.push(self.get_reactive());
            }
        }
        Ok(followed_path)
    }

    pub async fn wait_until_has_ball(&self) {
        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while !self.has_ball() {
            interval.tick().await;
        }
    }

    // what can you wait for a robot to do ?
    // - goto(pos)
    // - kick()
    // ??
}
