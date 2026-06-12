use std::{thread, time::Duration};

use reqwest::header::COOKIE;
use serde::{Serialize, de::DeserializeOwned};

use thiserror::Error;

use tungstenite::{
    Message, Utf8Bytes, WebSocket, client::IntoClientRequest, connect, stream::MaybeTlsStream,
};

pub use crate::experiment::websocket::*;
use crate::transport::Auth;

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

type Socket = WebSocket<MaybeTlsStream<std::net::TcpStream>>;
struct ConnectedSocket {
    socket: Socket,
    url: String,
    auth: Auth,
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
        let mut req = url
            .into_client_request()
            .expect("Should be able to create a client request from the URL");

        match &auth {
            Auth::None => {}
            Auth::SessionCookie(cookie) => {
                req.headers_mut().insert(COOKIE, cookie.parse().unwrap());
            }
            Auth::Bearer(token) => {
                req.headers_mut()
                    .insert("Authorization", format!("Bearer {token}").parse().unwrap());
            }
        }

        let (mut socket, _) =
            connect(req).map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        match socket.get_mut() {
            MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true),
            MaybeTlsStream::NativeTls(stream) => stream.get_mut().set_nonblocking(true),
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
        });
        Ok(())
    }

    fn reconnect(&mut self) -> Result<(), WebSocketError> {
        if let Some(socket) = self.state.take() {
            self.connect(&socket.url, &socket.auth)
        } else {
            Err(WebSocketError::CannotReconnect(
                "The websocket was never opened so it cannot be reconnected".to_string(),
            ))
        }
    }

    /// Sends a message over the WebSocket connection. This is a non-blocking call.
    /// If sending fails, it attempts to reconnect and resend the message.
    /// Returns an error if both attempts fail.
    pub fn send<I: Serialize>(&mut self, message: I) -> Result<(), WebSocketError> {
        let socket = self.active_socket()?;

        let json = serde_json::to_string(&message)
            .map_err(|e| WebSocketError::SerializationError(e.to_string()))?;

        match Self::attempt_send(socket, &json) {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::debug!("WebSocket send failed, attempting to reconnect...");
                thread::sleep(DEFAULT_RECONNECT_DELAY);
                self.reconnect()?;

                let socket = self.active_socket()?;
                Self::attempt_send(socket, &json)
            }
        }
    }

    /// Attempts to receive a message from the WebSocket. This is a non-blocking call.
    /// Returns `Ok(None)` if no message is available.
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
            Err(e) => Err(WebSocketError::ReceiveError(e.to_string())),
        }
    }

    fn attempt_send(socket: &mut Socket, payload: &str) -> Result<(), WebSocketError> {
        socket
            .send(Message::Text(Utf8Bytes::from(payload)))
            .map_err(|e| WebSocketError::SendError(e.to_string()))
    }

    /// Closes the WebSocket connection gracefully. This is a non-blocking call.
    pub fn close(&mut self) -> Result<(), WebSocketError> {
        let socket = self.active_socket()?;
        socket
            .close(None)
            .map_err(|e| WebSocketError::SendError(e.to_string()))
    }

    /// Waits until the WebSocket connection is fully closed. This is a blocking call that will return once the connection is closed.
    pub fn wait_until_closed(&mut self) -> Result<(), WebSocketError> {
        let socket = self.active_socket()?;
        match socket.get_mut() {
            MaybeTlsStream::Plain(stream) => stream.set_nonblocking(false),
            MaybeTlsStream::NativeTls(stream) => stream.get_mut().set_nonblocking(false),
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
