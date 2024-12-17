use crabe_async::vision::Vision;

#[tokio::main]
async fn main() {
    let mut vision = Vision::new("224.5.23.2", None, false);
    loop {
        for packet in vision.take_pending_packets().await {
            dbg!(packet);
        }
    }

    // let mut vision = MulticastUdpReceiver::new(Ipv4Addr::new(224, 5, 23, 2), 10020)
    //     .expect("Couldn't create vision's multicast udp receiver");
    // loop {
    //     match vision.receive::<SslWrapperPacket>().await {
    //         Ok(packet) => {
    //             dbg!(packet);
    //         }
    //         Err(e) => {
    //             eprintln!("failed to receive packet, reason: {:?}", e);
    //         }
    //     };
    // }
}
