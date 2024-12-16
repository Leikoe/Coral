use super::BUFFER_SIZE;
use prost::DecodeError;
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, Cursor};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket as StdUdpSocket};
use tokio::net::UdpSocket as TokioUdpSocket;

pub struct MulticastUdpReceiver {
    socket: TokioUdpSocket,
    buffer: [u8; BUFFER_SIZE],
}

#[derive(Debug)]
pub enum CreationError {
    SocketCreationError(io::Error),
    SocketReuseAddressError(io::Error),
    SocketNonblockingError(io::Error),
    SocketBindError(io::Error),
    SocketJoinMulticastError(io::Error),
}
use CreationError::*; // for readability in `MulticastUdpReceiver::new`

#[derive(Debug)]
pub enum ReceiveError {
    SocketReceiveError(io::Error),
    DecodeError(DecodeError),
}

impl MulticastUdpReceiver {
    pub fn new(ip: Ipv4Addr, port: u16) -> Result<Self, CreationError> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(SocketCreationError)?;
        socket
            .set_reuse_address(true)
            .map_err(SocketReuseAddressError)?;
        socket
            .set_nonblocking(true)
            .map_err(SocketNonblockingError)?;
        socket
            .bind(&SocketAddrV4::new(ip, port).into())
            .map_err(SocketBindError)?;
        socket
            .join_multicast_v4(&ip, &Ipv4Addr::UNSPECIFIED)
            .map_err(SocketJoinMulticastError)?;
        let std_socket: StdUdpSocket = socket.into();
        Ok(Self {
            socket: TokioUdpSocket::from_std(std_socket)
                .expect("failed to create a tokio UdpSocket from a std UdpSocket"),
            buffer: [0u8; BUFFER_SIZE],
        })
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
