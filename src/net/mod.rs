// https://stackoverflow.com/questions/77590001/can-i-convert-from-socket2socket-to-tokionettcpstream

use prost::{DecodeError, EncodeError};
use std::io;

pub mod multicast_receiver;
pub mod udp_transceiver;

const BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub enum ReceiveError {
    SocketReceiveError(io::Error),
    DecodeError(DecodeError),
}

#[derive(Debug)]
pub enum SendError {
    SocketSendError(io::Error),
    EncodeError(EncodeError),
}
