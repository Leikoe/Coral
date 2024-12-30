use crate::league_protocols::vision_packet::SslWrapperPacket;
use crate::net::multicast_receiver::MulticastUdpReceiver;
use std::net::Ipv4Addr;

const DEFAULT_VISION_IP: Ipv4Addr = Ipv4Addr::new(224, 5, 23, 2);
const DEFAULT_VISION_PORT_REAL: u16 = 10006;
const DEFAULT_VISION_PORT_SIM: u16 = 10020;

// TODO: Document
pub struct Vision {
    socket: MulticastUdpReceiver,
}

impl Vision {
    pub fn new(
        custom_vision_ip: Option<Ipv4Addr>,
        custom_vision_port: Option<u16>,
        real: bool,
    ) -> Self {
        let vision_ip = match custom_vision_ip {
            Some(custom_vision_ip) => custom_vision_ip,
            None => DEFAULT_VISION_IP,
        };

        let port = if let Some(port) = custom_vision_port {
            port
        } else if real {
            DEFAULT_VISION_PORT_REAL
        } else {
            DEFAULT_VISION_PORT_SIM
        };

        Self {
            socket: MulticastUdpReceiver::new(vision_ip, port)
                .expect("Failed to create vision receiver"),
        }
    }

    pub async fn receive(&mut self) -> Result<SslWrapperPacket, crate::net::ReceiveError> {
        self.socket.receive::<SslWrapperPacket>().await
    }

    // pub async fn take_pending_packets(&mut self) -> impl Iterator<Item = SslWrapperPacket> {
    //     let mut received = Vec::new();
    //     while let Ok(Ok(packet)) =
    //         tokio::time::timeout(RECEIVE_TIMEOUT, self.socket.receive::<SslWrapperPacket>()).await
    //     {
    //         received.push(packet);
    //     }
    //     received.into_iter()
    // }
}
