use crabe_async::vision::Vision;

#[tokio::main]
async fn main() {
    let mut vision = Vision::new(None, None, false);
    loop {
        match vision.receive().await {
            Ok(feedback) => {
                dbg!(feedback);
            }
            Err(e) => {
                eprintln!("error while receiving vision packet: {:?}", e);
            }
        }
    }
}
