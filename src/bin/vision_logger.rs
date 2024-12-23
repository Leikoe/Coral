use crabe_async::vision::Vision;

#[tokio::main]
async fn main() {
    let mut vision = Vision::new(None, None, false);
    while let Ok(packet) = vision.receive().await {
        dbg!(packet);
    }
}
