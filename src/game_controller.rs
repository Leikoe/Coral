use crate::league_protocols::game_controller_packet::Referee;
use crate::net::multicast_receiver::MulticastUdpReceiver;
use crate::net::ReceiveError;
use std::net::Ipv4Addr;

const DEFAULT_GC_IP: Ipv4Addr = Ipv4Addr::new(224, 5, 23, 1);
const DEFAUL_TGC_PORT: u16 = 10003;

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

    pub async fn receive(&mut self) -> Result<Referee, ReceiveError> {
        self.socket.receive::<Referee>().await
    }
}
