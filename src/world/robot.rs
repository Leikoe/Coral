use tokio::time::sleep;

use crate::{
    league_protocols::vision_packet::SslDetectionRobot,
    math::{angle_difference, Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext, Rect, Vec2},
    trajectories::{bangbang2d::BangBang2d, Trajectory},
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
    vel: Arc<Mutex<Vec2>>,
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
            vel: Arc::new(Mutex::new(Vec2::zero())),
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

    pub fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }

    pub fn set_pos(&mut self, pos: Point2) {
        let mut _pos = self.pos.lock().unwrap();
        *_pos = pos;
    }

    pub fn get_vel(&self) -> Vec2 {
        *self.vel.lock().unwrap()
    }

    pub fn set_vel(&mut self, vel: Vec2) {
        let mut _vel = self.vel.lock().unwrap();
        *_vel = vel;
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
        if detected_pos != self.get_pos() {
            self.set_vel(
                (detected_pos - self.get_pos()) * (1. / CONTROL_PERIOD.as_secs_f64()) as f32,
            );
        }
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

    pub fn pov(&self, pos: Point2) -> Point2 {
        let to_pos = self.to(&pos).get_reactive();
        let self_orientation = self.get_orientation();
        let inverse_orientation = -self_orientation;
        Point2::new(
            to_pos.x * inverse_orientation.cos() - to_pos.y * inverse_orientation.sin(),
            to_pos.y * inverse_orientation.cos() + to_pos.x * inverse_orientation.sin(),
        )
    }

    pub fn pov_vec(&self, vel: Vec2) -> Vec2 {
        let self_orientation = self.get_orientation();
        let inverse_orientation = -self_orientation;
        Vec2::new(
            vel.x * inverse_orientation.cos() - vel.y * inverse_orientation.sin(),
            vel.y * inverse_orientation.cos() + vel.x * inverse_orientation.sin(),
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

    fn is_free(&self, pos: Point2, world: &World, avoidance_mode: AvoidanceMode) -> bool {
        if let AvoidanceMode::None = avoidance_mode {
            return true;
        }

        let is_colliding_with_allies = world
            .team
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.get_id() != self.get_id()) // can't collide with myself
            .any(|r| r.collides_with_robot(pos));
        let is_colliding_with_ennemies = world
            .ennemies
            .lock()
            .unwrap()
            .values()
            .any(|r| r.collides_with_robot(pos));

        let is_colliding_with_a_robot = is_colliding_with_allies || is_colliding_with_ennemies;
        if let AvoidanceMode::AvoidRobots = avoidance_mode {
            return !is_colliding_with_a_robot;
        }

        let is_colliding_with_ball = pos.distance_to(&world.ball) < 0.2;
        return !is_colliding_with_a_robot && !is_colliding_with_ball;
    }

    pub fn is_a_valid_trajectory(
        &self,
        traj: &impl Trajectory<Point2, Vec2>,
        world: &World,
        avoidance_mode: AvoidanceMode,
    ) -> bool {
        const TIME_STEP: f64 = 0.200; // 200ms as per tiger's tdp
        let n_points_to_check: usize = (traj.get_total_runtime() / TIME_STEP) as usize;
        for i in 0..n_points_to_check {
            let t = i as f64 * TIME_STEP;
            let p = traj.get_position(t);
            if !self.is_free(p, world, avoidance_mode) {
                println!("collision at {}", t);
                return false;
            }
        }
        true
    }

    pub async fn goto_rrt<T: Reactive<Point2>>(
        &self,
        world: &World,
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
            let rect = *world.field.lock().unwrap(); // assume that the field won't change size during this path generation

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
        world: &World,
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

        let mut followed_path = vec![self.get_pos()];
        'traj: while self.get_pos().distance_to(&destination.get_reactive()) > IS_CLOSE_EPSILON
            && is_angle_right()
        {
            println!("trying to go to dest");
            let start = self.get_pos();
            let goal = destination.get_reactive();
            let field = world.field.lock().unwrap(); // assume that the field won't change size during this path generation

            let start_time = Instant::now();
            let path = rrt::dual_rrt_connect(
                &[start.x, start.y],
                &[goal.x, goal.y],
                |p| self.is_free(Point2::from_vec(p), world, avoidance_mode),
                || field.sample_inside().to_vec(),
                0.1,
                RRT_MAX_TRIES,
            )?;
            let path: Vec<Point2> = path
                .into_iter()
                .skip(1)
                .map(|p| Point2::from_vec(&p))
                .collect();

            // uncomment for straight line path testing (bang bang schedule testing)
            // let path = vec![self.get_pos(), destination.get_reactive()];
            let path_len = path.len();

            println!(
                "[TRACE - robot {} - goto_rrt] took {}ms to compute path",
                self.get_id(),
                start_time.elapsed().as_millis()
            );

            let mut last_p = path[0];
            for (i, p) in path.into_iter().enumerate() {
                println!("going to waypoint {}", i);
                'waypoint: loop {
                    let is_last_waypoint = i == path_len - 1;
                    // we put p further from the robot than it really is to make it go fast :)
                    let virtual_p = if is_last_waypoint {
                        p
                    } else {
                        p + last_p.to(p)
                    };
                    let traj = BangBang2d::new(
                        self.get_pos(),
                        self.get_vel(), // TODO: FIX THIS BY USING REAL VEL
                        virtual_p,
                        10.,
                        3.,
                        0.05,
                    );
                    let start = Instant::now();
                    while start.elapsed().as_secs_f64()
                        < traj.get_total_runtime() * if is_last_waypoint { 1. } else { 0.5 }
                    {
                        if !self.is_a_valid_trajectory(&traj, world, avoidance_mode) {
                            println!("detected collision on traj, generating a new path!");
                            continue 'traj; // generate a new path
                        }
                        let t = start.elapsed().as_secs_f64();
                        let v = self.pov_vec(traj.get_velocity(t));
                        let p = traj.get_position(t);
                        let p_diff = self.pov_vec(p - self.get_pos());
                        if p_diff.norm() > 0.5 {
                            println!("we fell off the traj!, trying again!");
                            continue 'waypoint; // generate a new traj from current pos to same waypoint
                        }
                        self.set_target_vel(v + p_diff * 0.5);
                        sleep(CONTROL_PERIOD).await;
                    }
                    break; // we're done with this waypoint
                }
                last_p = p;
            }
        }
        println!("arrived!");
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
