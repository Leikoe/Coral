use crabe_async::{
    controllers::{sim_controller::SimRobotController, RobotController},
    math::Point2,
    world::{Robot, TeamColor},
};

#[tokio::main]
async fn main() {
    let mut sim_controller = SimRobotController::new(TeamColor::Blue).await;

    let robot = Robot::new(0, Point2::zero(), 0.0);
    robot.set_target_angular_vel(1.);
    sim_controller
        .send_proper_command_for(vec![robot].into_iter())
        .await
        .expect("couldn't send the command to the simulator");
}
