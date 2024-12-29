use plotters::{
    chart::{ChartBuilder, LabelAreaPosition},
    prelude::{BitMapBackend, Circle, IntoDrawingArea},
    style::{BLUE, WHITE},
};
use tokio::{select, sync::Notify, time::sleep};

use crate::{
    league_protocols::vision_packet::SslDetectionRobot,
    math::{angle_difference, Point2, Reactive, ReactivePoint2Ext, ReactiveVec2Ext, Vec2},
    trajectories::{bangbang1d::BangBang1d, bangbang2d::BangBang2d, Trajectory},
    viewer::{self, ViewerObject},
    world::World,
    CONTROL_PERIOD, DETECTION_SCALING_FACTOR,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};

use super::{Ball, TeamColor};

pub type RobotId = u8;
const IS_CLOSE_EPSILON: f64 = 0.05;
const RRT_MAX_TRIES: usize = 1_000;

const GOTO_SPEED: f64 = 1.5;
const GOTO_ANGULAR_SPEED: f64 = 1.5;

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

#[derive(Clone)]
pub struct Robot<D: RobotData> {
    id: RobotId,
    color: TeamColor,
    pos: Arc<Mutex<Point2>>,
    vel: Arc<Mutex<Vec2>>,
    angular_vel: Arc<Mutex<f64>>,
    orientation: Arc<Mutex<f64>>,
    has_ball: Arc<Mutex<bool>>,
    last_update: Arc<Mutex<Option<f64>>>,
    internal_data: D,
}

#[derive(Clone, Default)]
pub struct AllyData {
    target_vel: Arc<Mutex<Vec2>>,
    target_angular_vel: Arc<Mutex<f64>>,
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
    pub fn new(id: RobotId, color: TeamColor, pos: Point2, orientation: f64) -> Self {
        Self {
            id,
            color,
            pos: Arc::new(Mutex::new(pos)),
            vel: Arc::new(Mutex::new(Vec2::zero())),
            angular_vel: Arc::new(Mutex::new(0.)),
            orientation: Arc::new(Mutex::new(orientation)),
            has_ball: Arc::new(Mutex::new(false)),
            last_update: Arc::new(Mutex::new(None)),
            internal_data: Default::default(),
        }
    }

    pub fn default_with_id(id: RobotId, color: TeamColor) -> Self {
        Self {
            id,
            color,
            pos: Default::default(),
            vel: Default::default(),
            angular_vel: Default::default(),
            orientation: Default::default(),
            has_ball: Default::default(),
            last_update: Arc::new(Mutex::new(None)),
            internal_data: Default::default(),
        }
    }

    pub fn get_id(&self) -> RobotId {
        self.id
    }

    pub fn get_color(&self) -> TeamColor {
        self.color
    }

    pub fn has_ball(&self) -> bool {
        *self.has_ball.lock().unwrap()
    }

    pub fn set_has_ball(&mut self, has_ball: bool) {
        *self.has_ball.lock().unwrap() = has_ball;
    }

    pub fn get_orientation(&self) -> f64 {
        *self.orientation.lock().unwrap()
    }

    pub fn get_pos(&self) -> Point2 {
        *self.pos.lock().unwrap()
    }

    pub fn set_pos(&mut self, pos: Point2) {
        *self.pos.lock().unwrap() = pos;
    }

    pub fn get_vel(&self) -> Vec2 {
        *self.vel.lock().unwrap()
    }

    pub fn set_vel(&mut self, vel: Vec2) {
        *self.vel.lock().unwrap() = vel;
    }

    pub fn get_angular_vel(&self) -> f64 {
        *self.angular_vel.lock().unwrap()
    }

    pub fn set_angular_vel(&mut self, vel: f64) {
        *self.angular_vel.lock().unwrap() = vel;
    }

    pub fn set_orientation(&mut self, orientation: f64) {
        *self.orientation.lock().unwrap() = orientation;
    }

    pub fn get_last_update(&self) -> Option<f64> {
        *self.last_update.lock().unwrap()
    }

    pub fn set_last_update(&mut self, last_update: f64) {
        *self.last_update.lock().unwrap() = Some(last_update);
    }

    pub fn update_from_packet(
        &mut self,
        detection: SslDetectionRobot,
        ball: &Ball,
        t_capture: f64,
    ) {
        let detected_pos = Point2::new(
            detection.x as f64 / DETECTION_SCALING_FACTOR,
            detection.y as f64 / DETECTION_SCALING_FACTOR,
        );
        let detectect_orientation = detection.orientation() as f64;
        if let Some(last_t) = self.get_last_update() {
            if last_t < t_capture {
                let dt = t_capture - last_t;
                self.set_vel((detected_pos - self.get_pos()) / dt); // TODO: remove f32 from the project :sob:
                self.set_angular_vel((detectect_orientation - self.get_orientation()) / dt);
            }
        }
        self.set_last_update(t_capture);
        self.set_pos(detected_pos);
        self.set_orientation(detectect_orientation);

        // viewer::render(ViewerObject::Robot {
        //     id: self.get_id(),
        //     color: self.get_color(),
        //     has_ball: self.has_ball(),
        //     pos: self.get_pos(),
        //     vel: self.get_vel(),
        // });

        // let has_ball = {
        //     let r_to_ball = self.to(ball);
        //     let is_facing_ball =
        //         angle_difference(r_to_ball.angle(), self.get_orientation()).abs()
        //             < 20.;
        //     is_facing_ball && (r_to_ball.norm() < 0.15) // TODO: stop the magic
        // };
        // self.set_has_ball(has_ball); // handled by robot feedback for allies, TODO: find a way for ennemies
    }

    fn collides_with_robot(&self, other_pos: Point2) -> bool {
        self.distance_to(&other_pos) < 0.3 // a robot is 10cm radius => 0.3 leaves 10cm between robots
    }

    pub fn pov(&self, pos_world: Point2) -> Point2 {
        let to_pos = self.to(&pos_world).get_reactive();
        let self_orientation = self.get_orientation();
        let inverse_orientation = -self_orientation;
        Point2::new(
            to_pos.x * inverse_orientation.cos() - to_pos.y * inverse_orientation.sin(),
            to_pos.y * inverse_orientation.cos() + to_pos.x * inverse_orientation.sin(),
        )
    }

    pub fn pov_vec(&self, vel_world: Vec2) -> Vec2 {
        let self_orientation = self.get_orientation();
        let inverse_orientation = -self_orientation;
        Vec2::new(
            vel_world.x * inverse_orientation.cos() - vel_world.y * inverse_orientation.sin(),
            vel_world.y * inverse_orientation.cos() + vel_world.x * inverse_orientation.sin(),
        )
    }

    pub fn orientation_diff_to(&self, target_orientation: f64) -> f64 {
        angle_difference(target_orientation, self.get_orientation())
    }
}

#[derive(Debug, Clone)]
pub enum GotoError {
    DestinationOccupiedError,
    NoPathFoundError(String),
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

    pub fn set_target_vel(&self, target_vel: Vec2) {
        *self.internal_data.target_vel.lock().unwrap() = target_vel;
    }

    pub fn get_target_angular_vel(&self) -> f64 {
        *self.internal_data.target_angular_vel.lock().unwrap()
    }

    pub fn set_target_angular_vel(&self, target_angular_vel: f64) {
        *self.internal_data.target_angular_vel.lock().unwrap() = target_angular_vel;
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
        const TIME_STEP: f64 = 0.050; // 200ms as per tiger's tdp
        let n_points_to_check: usize = (traj.get_total_runtime() / TIME_STEP) as usize;
        for i in 0..n_points_to_check {
            let t = i as f64 * TIME_STEP;
            let p = traj.get_position(t);
            if !self.is_free(p, world, avoidance_mode) {
                println!("[robot{}] collision at {}", self.get_id(), t);
                return false;
            }
        }
        true
    }

    fn make_bangbang2d_to(&self, dest: Point2) -> BangBang2d {
        BangBang2d::new(self.get_pos(), self.get_vel(), dest, 5., 4., 0.1)
    }

    async fn goto_straight<T: Reactive<Point2>>(
        &self,
        world: &World,
        destination: &T,
        angle: Option<f64>,
    ) {
        while !(self.get_pos().distance_to(&destination.get_reactive()) < IS_CLOSE_EPSILON
            && angle
                .map(|a| self.orientation_diff_to(a).abs() < 0.02)
                .unwrap_or(true)
            && self.get_vel().norm() < 0.02)
        {
            world.next_update().await;
            let traj = self.make_bangbang2d_to(destination.get_reactive());
            let v = self.pov_vec(traj.get_velocity(0.075));
            self.set_target_vel(v);

            if let Some(angle) = angle {
                // TODO: find a way to use BangBang1d for orientation
                let av = self.orientation_diff_to(angle) * GOTO_ANGULAR_SPEED;
                self.set_target_angular_vel(av);
            }
        }
    }

    async fn look_at<T: Reactive<Point2>>(&self, world: &World, destination: &T) {
        while !(self.orientation_diff_to(self.to(destination).angle()).abs() < 0.02) {
            // TODO: find a way to handle the angle
            self.set_target_angular_vel(
                self.orientation_diff_to(self.to(destination).angle()) * GOTO_ANGULAR_SPEED,
            );
            world.next_update().await;
        }
    }

    fn simplify_path(
        &self,
        world: &World,
        avoidance_mode: AvoidanceMode,
        path: Vec<Point2>,
    ) -> Vec<Point2> {
        let mut simplified_path = Vec::new();
        let path_len = path.len();
        let mut last_p = self.get_pos();
        for (i, p, is_last) in path
            .into_iter()
            .enumerate()
            .map(|(i, p)| (i, p, i == path_len - 1))
        {
            let t = BangBang2d::new(
                simplified_path.last().map(|p| *p).unwrap_or(self.get_pos()),
                Vec2::zero(),
                p,
                5.,
                4.,
                0.1,
            );
            let t_is_valid = self.is_a_valid_trajectory(&t, world, avoidance_mode);
            if is_last {
                simplified_path.push(p);
            }

            if !t_is_valid {
                println!("skipped to {}", i);
                simplified_path.push(last_p);
                last_p = p;
            } else {
                last_p = p;
                continue;
            }
        }
        simplified_path
    }

    pub async fn goto<T: Reactive<Point2>>(
        &self,
        world: &World,
        destination: &T,
        angle: Option<f64>,
        avoidance_mode: AvoidanceMode,
    ) -> Result<(), GotoError> {
        // if no avoidance_mode: fallback to Robot::goto
        if let AvoidanceMode::None = avoidance_mode {
            return Ok(self.goto_straight(world, destination, angle).await);
        }

        if !self.is_free(self.get_reactive(), world, avoidance_mode) {
            return Err(GotoError::DestinationOccupiedError);
        }

        'newpath: while self.get_pos().distance_to(&destination.get_reactive()) > IS_CLOSE_EPSILON
            || !angle
                .map(|a| self.orientation_diff_to(a).abs() < 0.02)
                .unwrap_or(true)
        {
            world.next_update().await;
            println!("[robot{}] trying to go to dest", self.get_id());
            let field = world.field.get_bounding_box(); // assume that the field won't change size during this path generation

            let traj = self.make_bangbang2d_to(destination.get_reactive());
            // if self.is_a_valid_trajectory(&traj, world, avoidance_mode) {
            //     println!("[robot{}] TRAJ WAS VALID, GOING FASSSTTTTT!", self.get_id());
            //     self.goto_straight(world, destination, angle).await;
            //     break;
            // }

            let start_time = Instant::now();
            let path = rrt::dual_rrt_connect(
                &self.get_pos().to_vec(),
                &destination.get_reactive().to_vec(),
                |p| self.is_free(Point2::from_vec(p), world, avoidance_mode),
                || field.sample_inside().to_vec(),
                0.1,
                RRT_MAX_TRIES,
            )
            .map_err(GotoError::NoPathFoundError)?;
            println!(
                "[TRACE - robot {} - goto_rrt] took {}ms to compute path",
                self.get_id(),
                start_time.elapsed().as_millis()
            );
            let path_without_current_pos: Vec<Point2> = path
                .into_iter()
                .skip(1)
                .map(|p| Point2::from_vec(&p))
                .collect();
            let simplified_path =
                self.simplify_path(world, avoidance_mode, path_without_current_pos);

            for (i, p) in simplified_path.into_iter().enumerate() {
                println!("going to point {}", i);
                while !(self.get_pos().distance_to(&p) < IS_CLOSE_EPSILON * 3.
                    && angle
                        .map(|a| self.orientation_diff_to(a).abs() < 0.02)
                        .unwrap_or(true))
                {
                    world.next_update().await;
                    let traj = self.make_bangbang2d_to(p);
                    if !self.is_a_valid_trajectory(&traj, world, avoidance_mode) {
                        println!("traj is now invalid, generating a new path!");
                        continue 'newpath;
                    }
                    let v = self.pov_vec(traj.get_velocity(0.075));
                    self.set_target_vel(v);
                    // viewer::render(ViewerObject::Point {
                    //     color: "grey",
                    //     pos: p,
                    // });
                    // viewer::render(ViewerObject::Point {
                    //     color: "red",
                    //     pos: destination.get_reactive(),
                    // });

                    if let Some(angle) = angle {
                        // TODO: find a way to use BangBang1d for orientation
                        let av = self.orientation_diff_to(angle) * GOTO_ANGULAR_SPEED;
                        self.set_target_angular_vel(av);
                    }
                }
            }
        }
        println!("arrived!");
        Ok(())
    }

    pub async fn wait_until_has_ball(&self) {
        let mut interval = tokio::time::interval(CONTROL_PERIOD);
        while !self.has_ball() {
            interval.tick().await;
        }
    }

    // after this call you should have the ball in your spinning dribbler
    pub async fn go_get_ball(&self, world: &World, ball: &Ball) {
        println!("go_get_ball()");
        self.enable_dribbler();
        let angle = self.to(ball).angle();
        while !self.has_ball() {
            select! {
                _ = self
                    .goto(
                        world,
                        ball,
                        Some(angle),
                        AvoidanceMode::None, // TODO: fix this when avoidance works again
                    ) => {}
                _ = self.wait_until_has_ball() => {
                    println!("we got ball");
                }
            };
        }
        sleep(Duration::from_millis(100)).await;
    }

    pub async fn pass_to(&self, world: &World, receiver: &AllyRobot) -> Result<(), String> {
        let to_receiver = self.to(receiver);
        let _ = self
            .goto(
                world,
                &self.get_pos(),
                Some(to_receiver.angle()),
                AvoidanceMode::AvoidRobots,
            )
            .await;
        sleep(Duration::from_millis(100)).await;
        let mut kick_cooldown = tokio::time::interval(CONTROL_PERIOD);
        while self.has_ball() {
            kick_cooldown.tick().await;
            self.kick();
        }
        match tokio::time::timeout(Duration::from_secs(1), receiver.wait_until_has_ball()).await {
            Ok(_) => {
                println!("[robot{}] passed!", self.get_id());
                Ok(())
            }
            Err(_) => Err("passed but didn't receive".to_string()),
        }
    }

    // what can you wait for a robot to do ?
    // - goto(pos)
    // - kick()
    // ??
}
