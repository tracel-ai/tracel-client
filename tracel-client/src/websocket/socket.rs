//! Websocket backends, one per target, behind a single [`Socket`] type.

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::sync::Once;

    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpStream;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::header::COOKIE;
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

    use crate::transport::Auth;
    use crate::websocket::WebSocketError;

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

    /// A websocket driven by tokio-tungstenite.
    pub struct Socket(WebSocketStream<MaybeTlsStream<TcpStream>>);

    impl Socket {
        /// Opens a connection to `url`, sending the session as the `Cookie`
        /// header of the handshake.
        pub async fn connect(url: &str, auth: &Auth) -> Result<Self, WebSocketError> {
            ensure_crypto_provider();

            let mut request = url.into_client_request().map_err(connection_error)?;
            if let Auth::SessionCookie(cookie) = auth {
                let cookie = cookie.parse().map_err(connection_error)?;
                request.headers_mut().insert(COOKIE, cookie);
            }

            let (stream, _) = connect_async(request).await.map_err(connection_error)?;
            Ok(Self(stream))
        }

        /// Sends `text` as a text frame.
        pub async fn send_text(&mut self, text: &str) -> Result<(), WebSocketError> {
            self.0
                .send(Message::Text(text.into()))
                .await
                .map_err(|e| WebSocketError::SendError(e.to_string()))
        }

        /// Waits for the next text frame; `None` once the connection is closed.
        pub async fn next_text(&mut self) -> Result<Option<String>, WebSocketError> {
            loop {
                match self.0.next().await {
                    None => return Ok(None),
                    Some(Ok(Message::Text(text))) => return Ok(Some(text.to_string())),
                    Some(Ok(Message::Binary(_) | Message::Frame(_))) => {
                        tracing::warn!("Received unexpected non-text message");
                    }
                    Some(Ok(Message::Ping(_) | Message::Pong(_) | Message::Close(_))) => {}
                    Some(Err(e)) => return Err(WebSocketError::ReceiveError(e.to_string())),
                }
            }
        }

        /// Sends a close frame and waits for the server to end the connection.
        pub async fn close(mut self) -> Result<(), WebSocketError> {
            self.0
                .close(None)
                .await
                .map_err(|e| WebSocketError::SendError(e.to_string()))?;

            while let Some(message) = self.0.next().await {
                message.map_err(|e| WebSocketError::ReceiveError(e.to_string()))?;
            }

            Ok(())
        }
    }

    fn connection_error(error: impl std::fmt::Display) -> WebSocketError {
        WebSocketError::ConnectionError(error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::pin::Pin;

    use futures_util::future::poll_fn;
    use futures_util::{Sink, SinkExt, StreamExt};
    use gloo_net::websocket::futures::WebSocket;
    use gloo_net::websocket::{Message, State, WebSocketError as BrowserError};
    use reqwest::Url;

    use crate::transport::Auth;
    use crate::websocket::WebSocketError;

    /// A websocket driven by the browser's `WebSocket` API.
    pub struct Socket(WebSocket);

    impl Socket {
        /// Opens a connection to `url`, passing the session token as the
        /// `token` query parameter.
        pub async fn connect(url: &str, auth: &Auth) -> Result<Self, WebSocketError> {
            let url = authenticated_url(url, auth)?;
            let mut socket = WebSocket::open(url.as_str()).map_err(connection_error)?;

            // The sink is ready once the `open` or `error` event has fired.
            poll_fn(|cx| Pin::new(&mut socket).poll_ready(cx))
                .await
                .map_err(connection_error)?;

            match socket.state() {
                State::Open => Ok(Self(socket)),
                _ => Err(connection_error(
                    "the browser could not open the connection",
                )),
            }
        }

        /// Sends `text` as a text frame.
        pub async fn send_text(&mut self, text: &str) -> Result<(), WebSocketError> {
            self.0
                .send(Message::Text(text.to_owned()))
                .await
                .map_err(|e| WebSocketError::SendError(e.to_string()))
        }

        /// Waits for the next text frame; `None` once the connection is closed.
        pub async fn next_text(&mut self) -> Result<Option<String>, WebSocketError> {
            loop {
                match self.0.next().await {
                    None => return Ok(None),
                    Some(Ok(Message::Text(text))) => return Ok(Some(text)),
                    Some(Ok(Message::Bytes(_))) => {
                        tracing::warn!("Received unexpected non-text message");
                    }
                    Some(Err(BrowserError::ConnectionClose(_))) => return Ok(None),
                    Some(Err(e)) => return Err(WebSocketError::ReceiveError(e.to_string())),
                }
            }
        }

        /// Starts the closing handshake, which the browser completes on its
        /// own once the socket is released.
        pub async fn close(self) -> Result<(), WebSocketError> {
            self.0
                .close(None, None)
                .map_err(|e| WebSocketError::SendError(e.to_string()))
        }
    }

    fn authenticated_url(url: &str, auth: &Auth) -> Result<Url, WebSocketError> {
        let mut url = Url::parse(url).map_err(connection_error)?;
        if let Auth::SessionCookie(cookie) = auth {
            url.query_pairs_mut()
                .append_pair("token", session_token(cookie));
        }
        Ok(url)
    }

    /// The value of the session cookie, whether it is a bare `name=value`
    /// pair or a full `Set-Cookie` line.
    fn session_token(cookie: &str) -> &str {
        cookie
            .split(';')
            .next()
            .and_then(|pair| pair.split_once('='))
            .map_or(cookie, |(_, value)| value.trim())
    }

    fn connection_error(error: impl std::fmt::Display) -> WebSocketError {
        WebSocketError::ConnectionError(error.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::Socket;
#[cfg(target_arch = "wasm32")]
pub use web::Socket;
