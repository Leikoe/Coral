use futures_util::{stream::FusedStream, SinkExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Notify,
};

use crate::{
    math::{Point2, Vec2},
    world::TeamColor,
};

const VIEWER_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const VIEWER_PORT: u16 = 8282;

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
pub struct ViewerFrame {
    objects: Vec<ViewerObject>,
}

static DRAWING_POOL: LazyLock<Mutex<HashMap<usize, ViewerObject>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn to_be_drawn_objects() -> usize {
    DRAWING_POOL.lock().unwrap().len()
}

fn get_frame() -> ViewerFrame {
    ViewerFrame {
        objects: DRAWING_POOL
            .lock()
            .unwrap()
            .values()
            .map(Clone::clone)
            .collect(),
    }
}

pub struct ViewerObjectGuard {
    id: usize,
}

impl Drop for ViewerObjectGuard {
    fn drop(&mut self) {
        let mut lock = DRAWING_POOL.lock().unwrap();
        lock.remove(&self.id);
    }
}

/// # Examples
///
/// Register the drawing by using `start_drawing()`
/// ```
/// use crabe_async::{math::Point2, viewer::{ViewerObject, start_drawing, to_be_drawn_objects}};
///
/// # assert_eq!(
/// #     to_be_drawn_objects(),
/// #     0,
/// #     "the drawing pool should be empty at init"
/// # );
///
/// let drawing_guard = start_drawing(ViewerObject::Point {
///     color: "blue",
///     pos: Point2::zero(),
/// });
/// assert_eq!(
///     to_be_drawn_objects(),
///     1,
///     "the drawing pool should now have the point"
/// );
/// drop(drawing_guard);
/// assert_eq!(
///     to_be_drawn_objects(),
///     0,
///     "the drawing pool should be empty after point guard was dropped"
/// );
/// ```
pub fn start_drawing(o: ViewerObject) -> ViewerObjectGuard {
    let uuid: usize = rand::random(); // TODO: replace this by a atomic usize
    let mut lock = DRAWING_POOL.lock().unwrap();
    lock.insert(uuid, o);
    ViewerObjectGuard { id: uuid }
}

pub async fn init() {
    let addr = SocketAddrV4::new(VIEWER_IP, VIEWER_PORT);
    let new_frame_notify = Arc::new(Notify::new());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    let new_frame_notify_clone = new_frame_notify.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(accept_connection(new_frame_notify_clone.clone(), stream));
        }
    });

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs_f32(1. / 60.));
        loop {
            interval.tick().await;
            new_frame_notify.notify_waiters();
        }
    });
}

async fn accept_connection(new_frame_notifier: Arc<Notify>, stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    while !ws_stream.is_terminated() {
        new_frame_notifier.notified().await;
        let frame = get_frame();
        let json_encoded_cmd = serde_json::to_string(&frame).unwrap();
        ws_stream
            .send(tokio_tungstenite::tungstenite::Message::text(
                json_encoded_cmd,
            ))
            .await
            .expect("couldn't send message to client");
    }
}
