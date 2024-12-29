use crabe_async::{
    controllers::{sim_controller::SimRobotController, RobotController},
    math::Point2,
    world::{AllyRobot, TeamColor},
};

#[tokio::main]
async fn main() {
    let team_color = TeamColor::Blue;
    let mut sim_controller = SimRobotController::new(team_color).await;

    let robot = AllyRobot::default_with_id(0, team_color);
    robot.set_target_angular_vel(1.);
    sim_controller
        .send_proper_command_for(vec![robot].into_iter())
        .await
        .expect("couldn't send the command to the simulator");
}
