use crate::league_protocols::game_controller_packet::Referee;
use crate::net::multicast_receiver::MulticastUdpReceiver;
use std::net::Ipv4Addr;
use std::time::Duration;

const DEFAULT_GC_IP: Ipv4Addr = Ipv4Addr::new(224, 5, 23, 1);
const DEFAUL_TGC_PORT: u16 = 10003;
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(10);

pub struct GameController {
    socket: MulticastUdpReceiver,
}

impl GameController {
    pub fn new(custom_gc_ip: Option<Ipv4Addr>, custom_gc_port: Option<u16>) -> Self {
        let ip = match custom_gc_ip {
            Some(custom_ip) => custom_ip,
            None => DEFAULT_GC_IP,
        };
        let port = match custom_gc_port {
            Some(custom_port) => custom_port,
            None => DEFAUL_TGC_PORT,
        };

        Self {
            socket: MulticastUdpReceiver::new(ip, port).expect("Failed to create GC receiver"),
        }
    }

    pub async fn take_pending_packets(&mut self) -> impl Iterator<Item = Referee> {
        let mut received = Vec::new();
        while let Ok(Ok(packet)) =
            tokio::time::timeout(RECEIVE_TIMEOUT, self.socket.receive::<Referee>()).await
        {
            received.push(packet);
        }
        received.into_iter()
    }
}
