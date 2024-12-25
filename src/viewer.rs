use futures_util::SinkExt;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Utf8Bytes;

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    for i in 0..100 {
        ws_stream
            .send(tokio_tungstenite::tungstenite::Message::Text(
                Utf8Bytes::from(format!("Hi {}!", i)),
            ))
            .await
            .expect("couldn't send message to client");
    }
}

pub async fn run_viewer_server_forever(ip: Ipv4Addr, port: u16) {
    let addr = SocketAddrV4::new(ip, port);
    let (viewer_tx, viewer_rx) = tokio::sync::mpsc::unbounded_channel();

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}
