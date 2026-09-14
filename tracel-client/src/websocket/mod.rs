use std::time::Duration;

use futures_timer::Delay;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

mod protocol;
mod socket;

pub use protocol::*;

use crate::transport::Auth;
use socket::Socket;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("Failed to connect WebSocket: {0}")]
    ConnectionError(String),
    #[error("WebSocket send error: {0}")]
    SendError(String),
    #[error("WebSocket receive error: {0}")]
    ReceiveError(String),
    #[error("WebSocket is not connected")]
    NotConnected,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

const RECONNECT_DELAY: Duration = Duration::from_millis(1000);

/// A JSON message channel over a websocket.
///
/// Every message travels as one text frame. Dropping a connected client ends
/// the connection without the closing handshake; [`close`] first to let the
/// server see a clean end.
///
/// [`close`]: WebSocketClient::close
pub struct WebSocketClient {
    socket: Option<Socket>,
    url: String,
    auth: Auth,
}

impl WebSocketClient {
    /// Opens a connection to `url`, authenticated as `auth`.
    ///
    /// Native targets send the session as the `Cookie` header of the
    /// handshake. A browser cannot set handshake headers, so on `wasm32` the
    /// session token travels as the `token` query parameter instead; accepting
    /// it there is a server-side coordination item (Q1).
    pub(crate) async fn connect(url: &str, auth: &Auth) -> Result<Self, WebSocketError> {
        let socket = Socket::connect(url, auth).await?;

        Ok(Self {
            socket: Some(socket),
            url: url.to_string(),
            auth: auth.clone(),
        })
    }

    /// Whether the connection is open.
    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    /// Sends `message` as a JSON text frame.
    ///
    /// A send that fails on an open connection is retried once over a fresh
    /// one; the error of that second attempt is returned.
    pub async fn send<I: Serialize>(&mut self, message: I) -> Result<(), WebSocketError> {
        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::SerializationError(e.to_string()))?;
        let socket = self.socket.as_mut().ok_or(WebSocketError::NotConnected)?;

        let Err(error) = socket.send_text(&json).await else {
            return Ok(());
        };

        tracing::debug!("WebSocket send failed ({error}), reconnecting");
        Delay::new(RECONNECT_DELAY).await;
        self.reconnect().await?.send_text(&json).await
    }

    async fn reconnect(&mut self) -> Result<&mut Socket, WebSocketError> {
        self.socket = None;
        let socket = Socket::connect(&self.url, &self.auth).await?;

        Ok(self.socket.insert(socket))
    }

    /// Waits for the next message from the server.
    ///
    /// Resolves to `None` once the connection is closed; frames that carry no
    /// text are skipped.
    pub async fn next<T: DeserializeOwned>(&mut self) -> Result<Option<T>, WebSocketError> {
        let Some(socket) = self.socket.as_mut() else {
            return Ok(None);
        };

        match socket.next_text().await? {
            Some(text) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| WebSocketError::SerializationError(e.to_string())),
            None => {
                self.socket = None;
                Ok(None)
            }
        }
    }

    /// Closes the connection, waiting for the closing handshake.
    ///
    /// Does nothing when the connection is already closed.
    pub async fn close(&mut self) -> Result<(), WebSocketError> {
        match self.socket.take() {
            Some(socket) => socket.close().await,
            None => Ok(()),
        }
    }
}
