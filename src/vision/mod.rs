use crate::league_protocols::vision_packet::SslWrapperPacket;
use crate::net::multicast_receiver::MulticastUdpReceiver;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;

const VISION_PORT_REAL: u16 = 10006;
const VISION_PORT_SIM: u16 = 10020;
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(5);

// TODO: Document
pub struct Vision {
    socket: MulticastUdpReceiver,
}

impl Vision {
    pub fn new(vision_ip: &str, vision_port: Option<u16>, real: bool) -> Self {
        let port = if let Some(port) = vision_port {
            port
        } else if real {
            VISION_PORT_REAL
        } else {
            VISION_PORT_SIM
        };

        let ipv4 =
            Ipv4Addr::from_str(vision_ip).expect("Failed to create an ipv4 address with the ip");

        Self {
            socket: MulticastUdpReceiver::new(ipv4, port)
                .expect("Failed to create vision receiver"),
        }
    }

    pub async fn take_pending_packets(&mut self) -> impl Iterator<Item = SslWrapperPacket> {
        let mut received = Vec::new();
        while let Ok(Ok(packet)) =
            tokio::time::timeout(RECEIVE_TIMEOUT, self.socket.receive::<SslWrapperPacket>()).await
        {
            received.push(packet);
        }
        received.into_iter()
    }
}
