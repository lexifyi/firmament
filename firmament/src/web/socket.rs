use axum::extract::ws::{Message, Utf8Bytes, WebSocket as AxumWebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    sync::mpsc,
    task::{AbortHandle, JoinHandle},
};

/// A WebSocket endpoint.
pub struct WebSocket {
    receiver: mpsc::Receiver<Utf8Bytes>,
    sender: mpsc::Sender<Utf8Bytes>,
    task: JoinHandle<()>,
}

/// A cloneable remote handle to a [`WebSocket`].
#[derive(Clone)]
pub struct WebSocketHandle {
    sender: mpsc::Sender<Utf8Bytes>,
    task: AbortHandle,
}

impl WebSocket {
    pub fn new(socket: AxumWebSocket) -> Self {
        let (in_tx, in_rx) = mpsc::channel(32);
        let (out_tx, out_rx) = mpsc::channel(32);

        let task = tokio::spawn(handle_socket(socket, in_tx, out_rx));

        Self {
            sender: out_tx,
            receiver: in_rx,
            task,
        }
    }

    /// Creates a cloneable handle for the socket which can be used to send
    /// messages or terminate the connection.
    #[inline]
    pub fn handle(&self) -> WebSocketHandle {
        WebSocketHandle {
            sender: self.sender.clone(),
            task: self.task.abort_handle(),
        }
    }

    /// Returns `false` if no further messages can be received.
    #[inline]
    pub fn can_recv(&self) -> bool {
        !self.receiver.is_closed()
    }

    /// Returns `false` if no further messages can be sent.
    #[inline]
    pub fn can_send(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Receives a deserialized JSON message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Option<T> {
        let json = self.recv_raw().await?;

        match serde_json::from_str(json.as_str()) {
            Ok(message) => Some(message),

            Err(err) => {
                #[cfg(debug_assertions)]
                log::error!("Failed to deserialize message. {err}");

                self.terminate();
                None
            }
        }
    }

    /// Receives a raw text message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    #[inline]
    pub fn recv_raw(&mut self) -> impl Future<Output = Option<Utf8Bytes>> {
        self.receiver.recv()
    }

    /// Sends a serialized JSON message on the socket.
    #[inline]
    pub fn send(&self, message: &impl Serialize) -> impl Future<Output = ()> {
        send(&self.sender, message)
    }

    /// Sends a raw text message on the socket.
    #[inline]
    pub fn send_raw(&self, text: impl Into<Utf8Bytes>) -> impl Future<Output = ()> {
        send_raw(&self.sender, text)
    }

    /// Terminates the connection immediately.
    #[inline]
    pub fn terminate(&self) {
        self.task.abort();
    }
}

impl WebSocketHandle {
    /// Sends a serialized JSON message on the socket.
    #[inline]
    pub fn send(&self, message: &impl Serialize) -> impl Future<Output = ()> {
        send(&self.sender, message)
    }

    /// Sends a raw text message on the socket.
    #[inline]
    pub fn send_raw(&self, text: impl Into<Utf8Bytes>) -> impl Future<Output = ()> {
        send_raw(&self.sender, text)
    }

    /// Terminates the connection immediately.
    #[inline]
    pub fn terminate(&self) {
        self.task.abort();
    }
}

/// Sends and receives WebSocket messages using MPSC channels.
///
/// This allows the socket to run on its own abortable task so that it can be
/// immediately disconnected on unexpected inputs, etc.
async fn handle_socket(
    socket: AxumWebSocket,
    in_chan: mpsc::Sender<Utf8Bytes>,
    out_chan: mpsc::Receiver<Utf8Bytes>,
) {
    let (sink, stream) = socket.split();

    // try_join allows both functions to run to completion unless one of them
    // returns an Err, which indicates that the socket should be dropped
    // immediately (because it e.g. sent an invalid message).
    let _ = tokio::try_join!(
        handle_sends(sink, out_chan),
        handle_receives(stream, in_chan)
    );
}

async fn handle_sends(
    mut sink: SplitSink<AxumWebSocket, Message>,
    mut chan: mpsc::Receiver<Utf8Bytes>,
) -> Result<(), ()> {
    while let Some(text) = chan.recv().await {
        if let Err(err) = sink.send(Message::Text(text)).await {
            #[cfg(debug_assertions)]
            log::error!("Send error: {err}");

            return Err(());
        }
    }

    Ok(())
}

async fn handle_receives(
    mut stream: SplitStream<AxumWebSocket>,
    chan: mpsc::Sender<Utf8Bytes>,
) -> Result<(), ()> {
    while let Some(item) = stream.next().await {
        match item {
            Ok(message) => match message {
                Message::Text(text) => {
                    if let Err(err) = chan.send(text).await {
                        #[cfg(debug_assertions)]
                        log::error!("Extra message: {:?}", err.0);

                        return Err(());
                    }
                }

                other => {
                    #[cfg(debug_assertions)]
                    log::error!("Non-text message: {other:?}");

                    return Err(());
                }
            },

            Err(err) => {
                #[cfg(debug_assertions)]
                log::error!("Receive error: {err}");

                return Err(());
            }
        };
    }

    Ok(())
}

#[inline]
#[track_caller]
fn send(sink: &mpsc::Sender<Utf8Bytes>, message: &impl Serialize) -> impl Future<Output = ()> {
    send_raw(
        sink,
        serde_json::to_string(message).expect("failed to serialize message to JSON"),
    )
}

#[inline]
fn send_raw(
    sink: &mpsc::Sender<Utf8Bytes>,
    json: impl Into<Utf8Bytes>,
) -> impl Future<Output = ()> {
    let json = json.into();

    async {
        let _ignore_error = sink.send(json).await;
    }
}
