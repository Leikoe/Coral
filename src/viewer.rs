//! Viewer abstraction.
//!
//! Provides an abstraction over a viewer client.  The api is guard based to be easily used in asynchronous code.
//!
//! # Examples
//!
//! If you want to draw a point to a target during a strategy:
//! ```
//! use crabe_async::{
//!     math::Point2,
//!     viewer::{ViewerObject, start_drawing, to_be_drawn_objects_count}
//! };
//! use tokio::time::{sleep, Duration};
//!
//! #[tokio::main]
//! async fn main() {
//!     let target_point = Point2::new(3., 0.);
//!     let target_point_drawing_guard = start_drawing(ViewerObject::Point {
//!         color: "red",
//!         pos: target_point,
//!     });
//!
//!     // This sleep will act as a 1s strategy.
//!     // During the sleep, the point will be drawn on each frame
//!     sleep(Duration::from_secs(1)).await;
//!
//!     // here the `target_point_drawing_guard` goes out of scope and gets dropped.
//!     // The point stops being drawn.
//! }
//! ```

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
    IgnoreMutexErr,
};

/// default viewer ip
const VIEWER_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// default viewer port
const VIEWER_PORT: u16 = 8282;

/// A shape that can be drawn on the viewer clients
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
    Segment {
        color: &'static str,
        start: Point2,
        end: Point2,
    },
}

/// A frame sent to each viewer client. It contains all the objects to be drawn during the frame.
#[derive(Serialize, Debug, Clone)]
pub struct ViewerFrame {
    objects: Vec<ViewerObject>,
}

static DRAWINGS_POOL: LazyLock<Mutex<HashMap<usize, ViewerObject>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Returns the number of `ViewerObject`s which should currently be drawn (the size of the internal drawings pool).
pub fn to_be_drawn_objects_count() -> usize {
    DRAWINGS_POOL.lock().unwrap_ignore_poison().len()
}

/// Returns a new `ViewerFrame` containing the `ViewerObject`s which should currently be drawn.
fn make_frame() -> ViewerFrame {
    ViewerFrame {
        objects: DRAWINGS_POOL
            .lock()
            .unwrap_ignore_poison()
            .values()
            .cloned()
            .collect(),
    }
}

/// A guard for a `ViewerObject` while it's being actively drawn.
/// When dropped, the ascociated `ViewerObject` stops being drawn.
pub struct ViewerObjectGuard {
    id: usize,
}

impl ViewerObjectGuard {
    pub fn update(&mut self, o: ViewerObject) {
        DRAWINGS_POOL
            .lock()
            .unwrap_ignore_poison()
            .insert(self.id, o);
    }
}

impl Drop for ViewerObjectGuard {
    fn drop(&mut self) {
        let mut lock = DRAWINGS_POOL.lock().unwrap_ignore_poison();
        lock.remove(&self.id);
    }
}

/// Makes the viewer start drawing the given `ViewerObject`.
/// Returns the `ViewerObjectGuard` associated with the object to be drawn.
/// The `ViewerObject` will be drawn each frame until the guard is dropped.
///
/// # Examples
///
/// Register the drawing by using `start_drawing()`
/// ```
/// use crabe_async::{math::Point2, viewer::{ViewerObject, start_drawing, to_be_drawn_objects_count}};
///
/// # assert_eq!(
/// #     to_be_drawn_objects_count(),
/// #     0,
/// #     "the drawing pool should be empty at init"
/// # );
///
/// let drawing_guard = start_drawing(ViewerObject::Point {
///     color: "blue",
///     pos: Point2::zero(),
/// });
/// assert_eq!(
///     to_be_drawn_objects_count(),
///     1,
///     "the drawing pool should now have the point"
/// );
/// drop(drawing_guard);
/// assert_eq!(
///     to_be_drawn_objects_count(),
///     0,
///     "the drawing pool should be empty after point guard was dropped"
/// );
/// ```
pub fn start_drawing(o: ViewerObject) -> ViewerObjectGuard {
    let uuid: usize = rand::random(); // TODO: replace this by a atomic usize
    let mut lock = DRAWINGS_POOL.lock().unwrap_ignore_poison();
    lock.insert(uuid, o);
    ViewerObjectGuard { id: uuid }
}

/// Starts the viewer server and the new frame thread.
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
        let frame = make_frame();
        let json_encoded_cmd =
            serde_json::to_string(&frame).expect("couldn't serialize `ViewerFrame`");
        ws_stream
            .send(tokio_tungstenite::tungstenite::Message::text(
                json_encoded_cmd,
            ))
            .await
            .expect("couldn't send message to client");
    }
}
