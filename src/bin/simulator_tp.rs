use crabe_async::{
    math::Point2, testing::simulation_control::SimulationController, world::TeamColor,
};

#[tokio::main]
async fn main() {
    let mut sim_controller = SimulationController::new().await;

    sim_controller
        .tp_robot(
            0,
            TeamColor::Blue,
            Some(Point2::new(3., 0.)),
            None,
            None,
            None,
        )
        .await
        .expect("couldn't tp robot in simulator");
}
