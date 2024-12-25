use futures_util::{stream::FusedStream, SinkExt};
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::OnceLock,
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::error::SendError,
        broadcast::{Receiver, Sender},
    },
};

use crate::{
    math::{Point2, Vec2},
    world::TeamColor,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ViewerObject {
    Robot {
        id: u8,
        color: TeamColor,
        has_ball: bool,
        pos: Point2,
        vel: Vec2,
    },
    Point {
        color: &'static str,
        pos: Point2,
    },
}

#[derive(Debug, Clone)]
pub enum ViewerCommand {
    Rerender,
    Render(ViewerObject),
}

#[derive(Serialize, Debug, Clone)]
pub struct ViewerFrame<'a> {
    objects: &'a [ViewerObject],
}

const VIEWER_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const VIEWER_PORT: u16 = 8282;
static VIEWER_TX: OnceLock<Sender<ViewerCommand>> = OnceLock::new();

#[derive(Debug)]
pub enum ViewerRenderError {
    ViewerUninitializedError,
    TxSendError(SendError<ViewerCommand>),
}

pub fn render(o: ViewerObject) -> Result<(), ViewerRenderError> {
    let tx = VIEWER_TX
        .get()
        .ok_or(ViewerRenderError::ViewerUninitializedError)?;

    tx.send(ViewerCommand::Render(o))
        .map_err(ViewerRenderError::TxSendError)?;
    Ok(())
}

pub fn rerender() -> Result<(), ViewerRenderError> {
    let tx = VIEWER_TX
        .get()
        .ok_or(ViewerRenderError::ViewerUninitializedError)?;

    tx.send(ViewerCommand::Rerender)
        .map_err(ViewerRenderError::TxSendError)?;
    Ok(())
}

pub async fn init() {
    if VIEWER_TX.get().is_none() {
        let addr = SocketAddrV4::new(VIEWER_IP, VIEWER_PORT);
        let (viewer_tx, viewer_rx) = tokio::sync::broadcast::channel::<ViewerCommand>(1024);

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        println!("Listening on: {}", addr);

        tokio::spawn(async move {
            let viewer_rx = viewer_rx;
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(accept_connection(viewer_rx.resubscribe(), stream));
            }
        });

        tokio::spawn(async {
            let mut interval = tokio::time::interval(Duration::from_secs_f32(1. / 60.));
            loop {
                interval.tick().await;
                rerender().expect("couldn't send the rerender command");
            }
        });

        VIEWER_TX.get_or_init(|| viewer_tx);
    }
}

async fn accept_connection(mut viewer_rx: Receiver<ViewerCommand>, stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let mut objects_buff = Vec::new();

    while !ws_stream.is_terminated() {
        if let Ok(cmd) = viewer_rx.recv().await {
            match cmd {
                ViewerCommand::Rerender => {
                    let frame = ViewerFrame {
                        objects: &objects_buff,
                    };
                    let json_encoded_cmd = serde_json::to_string(&frame).unwrap();
                    ws_stream
                        .send(tokio_tungstenite::tungstenite::Message::text(
                            json_encoded_cmd,
                        ))
                        .await
                        .expect("couldn't send message to client");
                    objects_buff.clear(); // once we sent all of the objects, we can discard them
                }
                ViewerCommand::Render(viewer_object) => {
                    objects_buff.push(viewer_object);
                }
            }
        }
    }
}
