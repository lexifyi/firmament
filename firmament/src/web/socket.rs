use axum::extract::ws::{Message, Utf8Bytes, WebSocket as AxumWebSocket};
use futures_util::{SinkExt, StreamExt, stream::SplitStream};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::mpsc;

/// A firament socket.
pub struct Socket {
    sender: Sender,
    stream: SplitStream<AxumWebSocket>,
}

/// A cloneable handle to a [`Socket`] that can be used to send messages or
/// close the connection.
pub struct Sender {
    sender: mpsc::UnboundedSender<Message>,
}

impl Socket {
    #[inline]
    pub fn new(socket: AxumWebSocket) -> Self {
        let (mut sink, stream) = socket.split();
        let (sender, mut receiver) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if sink.send(message).await.is_err() {
                    break;
                }
            }
        });

        Self {
            sender: Sender { sender },
            stream,
        }
    }

    /// Closes the socket gracefully.
    ///
    /// This is called automatically when the socket is dropped.
    #[inline]
    pub fn close(&mut self) {
        self.sender.close()
    }

    /// Receives a JSON message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Option<T> {
        while let Some(text) = self.recv_raw().await {
            match serde_json::from_str(text.as_str()) {
                Ok(message) => {
                    return Some(message);
                }

                Err(err) => {
                    #[cfg(debug_assertions)]
                    log::warn!("Failed to deserialize message: {err}");
                }
            }
        }

        None
    }

    /// Receives a raw text message from the socket.
    ///
    /// Returns `None` when the socket has been closed.
    pub async fn recv_raw(&mut self) -> Option<Utf8Bytes> {
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

    /// Sends a JSON message on the socket.
    ///
    /// Returns `false` if the socket is closed.
    #[inline]
    pub fn send(&mut self, message: &impl Serialize) -> bool {
        self.sender.send_json(message)
    }

    /// Sends a raw text message on the socket.
    ///
    /// Returns `false` if the socket is closed.
    #[inline]
    pub fn send_raw(&mut self, text: impl Into<Utf8Bytes>) -> bool {
        self.sender.send_text(text)
    }
}

impl Sender {
    /// Closes the socket gracefully.
    ///
    /// This is called automatically when the [`Socket`] is dropped.
    #[inline]
    pub fn close(&mut self) {
        let _ = self.sender.send(Message::Close(None));
    }

    /// Sends a JSON message on the socket.
    ///
    /// Returns `false` if the socket is closed.
    pub fn send_json(&mut self, message: &impl Serialize) -> bool {
        let json = serde_json::to_string(message).expect("failed to serialize message to JSON");

        self.send_text(json)
    }

    /// Sends a text message on the socket.
    ///
    /// Returns `false` if the socket is closed.
    #[inline]
    pub fn send_text(&mut self, text: impl Into<Utf8Bytes>) -> bool {
        self.sender.send(Message::Text(text.into())).is_ok()
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.close();
    }
}
