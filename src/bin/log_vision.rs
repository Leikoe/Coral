use crabe_async::vision::Vision;

#[tokio::main]
async fn main() {
    let mut vision = Vision::new(None, None, false);
    loop {
        for packet in vision.take_pending_packets().await {
            dbg!(packet);
        }
    }
}
