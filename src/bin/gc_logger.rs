use crabe_async::game_controller::GameController;

#[tokio::main]
async fn main() {
    let mut vision = GameController::new(None, None);
    loop {
        for packet in vision.take_pending_packets().await {
            dbg!(packet);
        }
    }
}
