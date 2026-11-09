use axum::extract::ws::{Message, Utf8Bytes, WebSocket as AxumWebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Serialize, de::DeserializeOwned};

/// A WebSocket endpoint.
pub struct WebSocket {
    receiver: WebSocketReceiver,
    sender: WebSocketSender,
}

/// The receiver half of a WebSocket returned from [`WebSocket::split()`].
pub struct WebSocketReceiver {
    stream: SplitStream<AxumWebSocket>,
}

/// The sender half of a WebSocket returned from [`WebSocket::split()`].
pub struct WebSocketSender {
    sink: SplitSink<AxumWebSocket, Message>,
}

impl WebSocket {
    #[inline]
    pub fn new(socket: AxumWebSocket) -> Self {
        let (sink, stream) = socket.split();
        let receiver = WebSocketReceiver { stream };
        let sender = WebSocketSender { sink };

        Self { receiver, sender }
    }

    /// Closes the socket gracefully.
    #[inline]
    pub fn close(&mut self) -> impl Future<Output = ()> {
        self.sender.close()
    }

    /// Receives a JSON message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    #[inline]
    pub fn recv_json<T: DeserializeOwned>(&mut self) -> impl Future<Output = Option<T>> {
        self.receiver.recv_json()
    }

    /// Receives a text message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    #[inline]
    pub fn recv_text(&mut self) -> impl Future<Output = Option<Utf8Bytes>> {
        self.receiver.recv_text()
    }

    /// Sends a JSON message on the socket.
    #[inline]
    pub fn send_json(&mut self, message: &impl Serialize) -> impl Future<Output = bool> {
        self.sender.send_json(message)
    }

    /// Sends a text message on the socket.
    #[inline]
    pub fn send_text(&mut self, text: impl Into<Utf8Bytes>) -> impl Future<Output = bool> {
        self.sender.send_text(text)
    }

    /// Splits this WebSocket into sender and receiver halves which can be used
    /// concurrenctly.
    #[inline]
    pub fn split(self) -> (WebSocketSender, WebSocketReceiver) {
        (self.sender, self.receiver)
    }
}

impl WebSocketReceiver {
    /// Receives a JSON message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    pub async fn recv_json<T: DeserializeOwned>(&mut self) -> Option<T> {
        while let Some(text) = self.recv_text().await {
            match serde_json::from_str(text.as_str()) {
                Ok(message) => {
                    return Some(message);
                }

                Err(err) => {
                    #[cfg(debug_assertions)]
                    log::warn!("Failed to deserialize message: {err}");
                    continue;
                }
            }
        }

        None
    }

    /// Receives a text message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    pub async fn recv_text(&mut self) -> Option<Utf8Bytes> {
        while let Some(result) = self.stream.next().await {
            match result {
                Ok(message) => match message {
                    Message::Text(text) => {
                        return Some(text);
                    }

                    Message::Close(_) => {
                        return None;
                    }

                    other => {
                        #[cfg(debug_assertions)]
                        log::warn!("Ignored message: {other:?}");
                    }
                },

                Err(err) => {
                    #[cfg(debug_assertions)]
                    log::error!("Receive error: {err}");
                }
            }
        }

        None
    }
}

impl WebSocketSender {
    /// Closes the socket gracefully.
    #[inline]
    pub async fn close(&mut self) {
        let _ = self.sink.send(Message::Close(None)).await;
    }

    /// Sends a JSON message on the socket.
    pub fn send_json(&mut self, message: &impl Serialize) -> impl Future<Output = bool> {
        let json = serde_json::to_string(message).expect("failed to serialize message to JSON");

        self.send_text(json)
    }

    /// Sends a text message on the socket.
    #[inline]
    pub fn send_text(&mut self, text: impl Into<Utf8Bytes>) -> impl Future<Output = bool> {
        let text = text.into();

        async { self.sink.send(Message::Text(text)).await.is_ok() }
    }
}
