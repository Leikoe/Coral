use super::{ReceiveError, SendError, BUFFER_SIZE};
use std::io::{self, Cursor};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::net::UdpSocket;

pub struct UdpTransceiver {
    socket: UdpSocket,
    buffer: [u8; BUFFER_SIZE],
}

pub enum UdpTransceiverCreationError {
    SocketBindError(io::Error),
    SocketConnectError(io::Error),
}

impl UdpTransceiver {
    pub async fn new(ip: Ipv4Addr, port: u16) -> Result<Self, UdpTransceiverCreationError> {
        let socket = UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0))
            .await
            .map_err(UdpTransceiverCreationError::SocketBindError)?;
        socket
            .connect(SocketAddrV4::new(ip, port))
            .await
            .map_err(UdpTransceiverCreationError::SocketConnectError)?;
        let buffer = [0u8; BUFFER_SIZE];

        Ok(Self { socket, buffer })
    }

    pub async fn send<T: prost::Message + Default>(&self, packet: T) -> Result<usize, SendError> {
        let mut buf = Vec::with_capacity(packet.encoded_len());
        packet.encode(&mut buf).map_err(SendError::EncodeError)?;
        let data = &buf[0..packet.encoded_len()];
        self.socket
            .send(data)
            .await
            .map_err(SendError::SocketSendError)
    }

    pub async fn receive<T: prost::Message + Default>(&mut self) -> Result<T, ReceiveError> {
        let received_bytes_count = self
            .socket
            .recv(&mut self.buffer)
            .await
            .map_err(ReceiveError::SocketReceiveError)?;
        T::decode(Cursor::new(&self.buffer[0..received_bytes_count]))
            .map_err(ReceiveError::DecodeError)
    }
}
