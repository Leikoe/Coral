use crabe_async::{controllers::sim_controller::SimRobotController, world::TeamColor};

#[tokio::main]
async fn main() {
    let mut sim_controller = SimRobotController::new(TeamColor::Blue).await;
    loop {
        match sim_controller.receive_feedback().await {
            Ok(feedback) => {
                dbg!(feedback);
            }
            Err(e) => {
                eprintln!("error while receiving robot feedback: {:?}", e);
            }
        }
    }
}
