use std::{net::TcpStream, sync::Arc, thread, time::Duration};

use reqwest::header::COOKIE;
use rustls::{ClientConfig, RootCertStore};
use serde::{Serialize, de::DeserializeOwned};

use thiserror::Error;

use tungstenite::{
    Connector, HandshakeError, Message, Utf8Bytes, WebSocket,
    client::{IntoClientRequest, uri_mode},
    client_tls_with_config,
    error::UrlError,
    handshake::client::{Request, Response},
    http::{Uri, request::Parts},
    stream::{MaybeTlsStream, Mode},
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

/// Matches the limit `tungstenite::connect` applies.
const MAX_REDIRECTS: u8 = 3;

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

        let (mut socket, _) = Self::handshake(req)?;

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
        });
        Ok(())
    }

    /// Connects and performs the websocket handshake, following redirects like
    /// `tungstenite::connect` does.
    ///
    /// This exists instead of `tungstenite::connect` only because that function hands `None` to
    /// `client_tls_with_config`, leaving the choice of crypto provider to rustls. See
    /// [`tls_config`] for why that choice has to be made here.
    fn handshake(request: Request) -> Result<(Socket, Response), WebSocketError> {
        let config = tls_config()?;
        let (parts, _) = request.into_parts();
        let mut uri = parts.uri.clone();

        for attempt in 0..=MAX_REDIRECTS {
            let request = request_with_uri(&parts, &uri);

            match Self::try_handshake(request, Connector::Rustls(config.clone())) {
                Err(tungstenite::Error::Http(response))
                    if response.status().is_redirection() && attempt < MAX_REDIRECTS =>
                {
                    let location = response.headers().get("Location").ok_or_else(|| {
                        WebSocketError::ConnectionError(
                            "redirected without a `Location` header".to_string(),
                        )
                    })?;

                    uri = location
                        .to_str()
                        .map_err(|e| {
                            WebSocketError::ConnectionError(format!("invalid `Location`: {e}"))
                        })?
                        .parse::<Uri>()
                        .map_err(|e| {
                            WebSocketError::ConnectionError(format!("invalid `Location`: {e}"))
                        })?;

                    tracing::debug!("WebSocket redirected to {uri}");
                }
                other => {
                    return other.map_err(|e| WebSocketError::ConnectionError(e.to_string()));
                }
            }
        }

        Err(WebSocketError::ConnectionError(format!(
            "exceeded {MAX_REDIRECTS} redirects"
        )))
    }

    fn try_handshake(
        request: Request,
        connector: Connector,
    ) -> Result<(Socket, Response), tungstenite::Error> {
        let uri = request.uri();
        let mode = uri_mode(uri)?;

        let host = uri.host().ok_or(UrlError::NoHostName)?;
        // An IPv6 host arrives bracketed, which `to_socket_addrs` will not parse.
        let host = host
            .strip_prefix('[')
            .and_then(|host| host.strip_suffix(']'))
            .unwrap_or(host);
        let port = uri.port_u16().unwrap_or(match mode {
            Mode::Plain => 80,
            Mode::Tls => 443,
        });

        let stream = TcpStream::connect((host, port))?;
        stream.set_nodelay(true)?;

        // A `ws://` request ignores the connector, so passing one is safe for both modes.
        client_tls_with_config(request, stream, None, Some(connector)).map_err(|e| match e {
            HandshakeError::Failure(e) => e,
            HandshakeError::Interrupted(_) => {
                unreachable!("a blocking handshake cannot be interrupted")
            }
        })
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

/// Builds the TLS configuration for `wss://` connections, naming the crypto provider explicitly.
///
/// rustls only infers a provider when exactly one of `ring` and `aws-lc-rs` is linked. That holds
/// for this crate on its own — reqwest brings in aws-lc-rs and nothing else — but not for every
/// dependent: burn reaches this crate alongside an older reqwest that brings in ring, and the two
/// unify onto one rustls with both providers enabled. Inference then panics rather than picking,
/// taking the handshake down with it.
///
/// Naming aws-lc-rs keeps the handshake working whatever else a dependent links, and agrees with
/// the provider reqwest already selects here, so a process ends up using just the one.
///
/// The root store mirrors tungstenite's own `rustls-tls-webpki-roots` behaviour, which is what
/// this replaces.
fn tls_config() -> Result<Arc<ClientConfig>, WebSocketError> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| WebSocketError::ConnectionError(format!("failed to configure TLS: {e}")))?
        .with_root_certificates(roots)
        .with_no_client_auth();

    Ok(Arc::new(config))
}

/// Rebuilds a request against `uri`, carrying the original headers over to a redirect target.
fn request_with_uri(parts: &Parts, uri: &Uri) -> Request {
    let mut builder = Request::builder()
        .uri(uri.clone())
        .method(parts.method.clone())
        .version(parts.version);

    if let Some(headers) = builder.headers_mut() {
        *headers = parts.headers.clone();
    }

    builder
        .body(())
        .expect("a request rebuilt from valid parts should be valid")
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        _ = self.close();
    }
}
