use crabe_async::game_controller::GameController;

#[tokio::main]
async fn main() {
    let mut gc = GameController::new(None, None);
    loop {
        match gc.receive().await {
            Ok(feedback) => {
                dbg!(feedback);
            }
            Err(e) => {
                eprintln!("error while receiving vision packet: {:?}", e);
            }
        }
    }
}
