use crabe_async::{controllers::sim_controller::{SimRobotController}, world::TeamColor};

#[tokio::main]
async fn main() {
    let mut sim_controller = SimRobotController::new(TeamColor::Blue).await;
    while let Ok(feedback) = sim_controller. {

    }
}
