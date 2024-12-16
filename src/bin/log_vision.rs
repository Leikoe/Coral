use crabe_async::{
    league_protocols::vision_packet::SslWrapperPacket,
    net::multicast_receiver::MulticastUdpReceiver,
};
use std::net::Ipv4Addr;

#[tokio::main]
async fn main() {
    let mut vision = MulticastUdpReceiver::new(Ipv4Addr::new(224, 5, 23, 2), 10020)
        .expect("Couldn't create vision's multicast udp receiver");
    loop {
        match vision.receive::<SslWrapperPacket>().await {
            Ok(packet) => {
                dbg!(packet);
            }
            Err(e) => {
                eprintln!("failed to receive packet, reason: {:?}", e);
            }
        };
    }
}
