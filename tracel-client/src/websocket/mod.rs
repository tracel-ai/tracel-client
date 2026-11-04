use std::{sync::Once, thread, time::Duration};

use reqwest::header::AUTHORIZATION;
use serde::{Serialize, de::DeserializeOwned};

use thiserror::Error;

use tungstenite::{
    Bytes, Message, Utf8Bytes, WebSocket, client::IntoClientRequest, connect,
    stream::MaybeTlsStream,
};

mod protocol;

pub use protocol::*;

use crate::transport::{Auth, Presented};

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum WebSocketError {
    #[error("Failed to connect WebSocket: {0}")]
    ConnectionError(String),
    #[error("WebSocket send error: {0}")]
    SendError(String),
    #[error("WebSocket receive error: {0}")]
    ReceiveError(String),
    #[error("WebSocket is not connected")]
    NotConnected,
    #[error("WebSocket cannot reconnect: {0}")]
    CannotReconnect(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

const DEFAULT_RECONNECT_DELAY: Duration = Duration::from_millis(1000);

static INSTALL_CRYPTO_PROVIDER: Once = Once::new();

/// Ensures a process-level rustls [`CryptoProvider`] is installed.
///
/// Multiple rustls crypto backends (`ring` and `aws-lc-rs`) can end up compiled
/// in through feature unification with other crates in the dependency graph.
/// When more than one is present, rustls cannot pick a default automatically and
/// tungstenite's `ClientConfig::builder()` panics. Installing the `ring` provider
/// explicitly removes that ambiguity. The `Once` makes this idempotent, and any
/// error means another provider is already installed, which is fine.
fn ensure_crypto_provider() {
    INSTALL_CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

type Socket = WebSocket<MaybeTlsStream<std::net::TcpStream>>;
struct ConnectedSocket {
    socket: Socket,
    url: String,
    auth: Auth,
    closed_on_purpose: bool,
}

#[derive(Default)]
pub struct WebSocketClient {
    state: Option<ConnectedSocket>,
}

impl WebSocketClient {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.state.is_some()
    }

    pub(crate) fn connect(&mut self, url: &str, auth: &Auth) -> Result<(), WebSocketError> {
        ensure_crypto_provider();

        let mut socket = open_socket(url, auth)?;

        match socket.get_mut() {
            MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true),
            MaybeTlsStream::Rustls(stream) => stream.sock.set_nonblocking(true),
            _ => unimplemented!("Other TLS streams are not supported"),
        }
        .map_err(|e| {
            WebSocketError::ConnectionError(format!("Failed to set non-blocking mode: {e}"))
        })?;

        let url = url.to_string();
        self.state = Some(ConnectedSocket {
            socket,
            url,
            auth: auth.clone(),
            closed_on_purpose: false,
        });
        Ok(())
    }

    /// The dead socket stays in place until a new one replaces it, so a failed attempt
    /// leaves the URL and credentials for the next one.
    fn reconnect(&mut self) -> Result<(), WebSocketError> {
        let Some(previous) = self.state.as_ref() else {
            return Err(WebSocketError::CannotReconnect(
                "The websocket was never opened so it cannot be reconnected".to_string(),
            ));
        };
        let (url, auth) = (previous.url.clone(), previous.auth.clone());
        self.connect(&url, &auth)
    }

    /// Sends a message over the WebSocket connection. This is a non-blocking call.
    /// If sending fails, it attempts to reconnect and resend the message.
    /// Returns an error if both attempts fail.
    pub fn send<I: Serialize>(&mut self, message: I) -> Result<(), WebSocketError> {
        self.active_socket()?;

        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::SerializationError(e.to_string()))?;

        self.send_frame_reconnecting_once_if_it_fails(Message::Text(Utf8Bytes::from(json)))
    }

    /// Sends a ping so that proxies between here and the server, which close a connection
    /// that carries nothing for a while, keep this one open while the caller has nothing to
    /// say. The server's pong is consumed by [`receive`](Self::receive). Like
    /// [`send`](Self::send), it reconnects once if the ping cannot be written.
    pub fn send_keepalive_ping(&mut self) -> Result<(), WebSocketError> {
        self.send_frame_reconnecting_once_if_it_fails(Message::Ping(Bytes::new()))
    }

    fn send_frame_reconnecting_once_if_it_fails(
        &mut self,
        frame: Message,
    ) -> Result<(), WebSocketError> {
        let socket = self.active_socket()?;

        match Self::attempt_send(socket, frame.clone()) {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::debug!("WebSocket send failed, attempting to reconnect...");
                thread::sleep(DEFAULT_RECONNECT_DELAY);
                self.reconnect()?;

                let socket = self.active_socket()?;
                Self::attempt_send(socket, frame)
            }
        }
    }

    /// Attempts to receive a message from the WebSocket. This is a non-blocking call.
    /// Returns `Ok(None)` if no message is available.
    ///
    /// A connection lost without [`close`](Self::close) having been called is reopened
    /// before this returns `Ok(None)`; the error is returned only when reopening fails.
    pub fn receive<T: DeserializeOwned>(&mut self) -> Result<Option<T>, WebSocketError> {
        let socket = self.active_socket()?;

        match socket.read() {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    let deserialized: T = serde_json::from_str(&text)
                        .map_err(|e| WebSocketError::SerializationError(e.to_string()))?;
                    Ok(Some(deserialized))
                }
                Message::Binary(_) => {
                    tracing::warn!("Received unexpected binary message");
                    Ok(None)
                }
                Message::Ping(_) | Message::Pong(_) | Message::Close(_) => Ok(None),
                Message::Frame(frame) => {
                    tracing::warn!("Received unexpected frame message: {:?}", frame);
                    Ok(None)
                }
            },
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No messages available
                Ok(None)
            }
            Err(e) if self.was_closed_on_purpose() => {
                Err(WebSocketError::ReceiveError(e.to_string()))
            }
            Err(e) => {
                tracing::debug!(error = %e, "WebSocket connection lost, attempting to reconnect...");
                thread::sleep(DEFAULT_RECONNECT_DELAY);
                self.reconnect()?;
                Ok(None)
            }
        }
    }

    fn attempt_send(socket: &mut Socket, frame: Message) -> Result<(), WebSocketError> {
        socket
            .send(frame)
            .map_err(|e| WebSocketError::SendError(e.to_string()))
    }

    /// Closes the WebSocket connection gracefully. This is a non-blocking call.
    pub fn close(&mut self) -> Result<(), WebSocketError> {
        if let Some(connected) = self.state.as_mut() {
            connected.closed_on_purpose = true;
        }
        let socket = self.active_socket()?;
        socket
            .close(None)
            .map_err(|e| WebSocketError::SendError(e.to_string()))
    }

    fn was_closed_on_purpose(&self) -> bool {
        self.state
            .as_ref()
            .is_some_and(|connected| connected.closed_on_purpose)
    }

    /// Waits until the WebSocket connection is fully closed. This is a blocking call that will return once the connection is closed.
    pub fn wait_until_closed(&mut self) -> Result<(), WebSocketError> {
        let socket = self.active_socket()?;
        match socket.get_mut() {
            MaybeTlsStream::Plain(stream) => stream.set_nonblocking(false),
            MaybeTlsStream::Rustls(stream) => stream.get_mut().set_nonblocking(false),
            _ => unimplemented!("Other TLS streams are not supported"),
        }
        .map_err(|e| {
            WebSocketError::ConnectionError(format!("Failed to set blocking mode: {e}"))
        })?;
        loop {
            match socket.read() {
                Ok(_) => {}
                Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => {
                    tracing::debug!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket read error while waiting until closed: {e}");
                    return Err(WebSocketError::SendError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    fn active_socket(&mut self) -> Result<&mut Socket, WebSocketError> {
        if let Some(socket) = self.state.as_mut() {
            Ok(&mut socket.socket)
        } else {
            Err(WebSocketError::NotConnected)
        }
    }
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        _ = self.close();
    }
}

/// A 401 renews an app session's access token once before the handshake fails.
fn open_socket(url: &str, auth: &Auth) -> Result<Socket, WebSocketError> {
    let presented = auth
        .present()
        .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;
    match handshake(url, &presented) {
        Err(tungstenite::Error::Http(response))
            if response.status() == tungstenite::http::StatusCode::UNAUTHORIZED =>
        {
            match auth
                .present_after_refusal(&presented)
                .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?
            {
                Some(renewed) => handshake(url, &renewed),
                None => Err(tungstenite::Error::Http(response)),
            }
        }
        outcome => outcome,
    }
    .map_err(|e| WebSocketError::ConnectionError(e.to_string()))
}

fn handshake(url: &str, presented: &Presented) -> Result<Socket, tungstenite::Error> {
    let mut request = url.into_client_request()?;
    if let Some(value) = presented.header() {
        request.headers_mut().insert(AUTHORIZATION, value.clone());
    }
    connect(request).map(|(socket, _)| socket)
}
