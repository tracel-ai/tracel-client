use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::credentials::TracelCredentials;
use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};
use crate::transport::{ApiTransport, Auth};

/// Name of the session cookie the server authenticates requests with.
const SESSION_COOKIE: &str = "id";

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        match error.status() {
            Some(status) => ClientError::ApiError {
                status,
                body: ApiErrorBody {
                    code: ApiErrorCode::Unknown,
                    message: error.to_string(),
                },
            },
            None => ClientError::UnknownError(error.to_string()),
        }
    }
}

/// A client for making HTTP requests to the Tracel API.
///
/// The client can be used to interact with the Tracel server, such as creating and starting experiments, saving and loading checkpoints, and uploading logs.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) transport: ApiTransport,
    pub(crate) env: Option<Env>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Env {
    Production,
    Staging(u8),
    Development,
}

impl Env {
    pub fn get_url(&self) -> Url {
        match self {
            Env::Production => Url::parse("https://console.tracel.ai/api/").unwrap(),
            Env::Staging(version) => {
                Url::parse(&format!("https://s{}-console.tracel.ai/api/", version)).unwrap()
            }
            Env::Development => Url::parse("http://localhost:9001/").unwrap(),
        }
    }
}

impl Client {
    /// Create a new client for the given environment, authenticated with the given credentials.
    ///
    /// Fails when the credentials are rejected, or when the server answers without a session
    /// cookie.
    pub fn new(env: Env, credentials: &TracelCredentials) -> Result<Self, ClientError> {
        let mut client = Client {
            transport: ApiTransport::new(env.get_url()),
            env: Some(env),
        };

        let cookie = client.login(credentials)?;
        client.transport.set_auth(Auth::SessionCookie(cookie));
        Ok(client)
    }

    /// Create a new client for an arbitrary base URL, authenticated with the given credentials.
    ///
    /// Use this to target a deployment [`Env`] does not name, such as a local devstack. Fails
    /// when the credentials are rejected, or when the server answers without a session cookie.
    pub fn from_url(url: Url, credentials: &TracelCredentials) -> Result<Self, ClientError> {
        let mut client = Client {
            transport: ApiTransport::new(url),
            env: None,
        };

        let cookie = client.login(credentials)?;
        client.transport.set_auth(Auth::SessionCookie(cookie));
        Ok(client)
    }

    /// Create a client for the given environment from a session token obtained elsewhere.
    ///
    /// No request is made: the token is trusted until the server rejects it with
    /// [`ClientError::Unauthorized`]. Use [`Client::get_current_user`] to check whether the
    /// session is still alive.
    pub fn with_session_token(env: Env, session_token: impl AsRef<str>) -> Self {
        let transport =
            ApiTransport::new(env.get_url()).with_auth(session_auth(session_token.as_ref()));

        Client {
            transport,
            env: Some(env),
        }
    }

    /// Create a client for an arbitrary base URL from a session token obtained elsewhere.
    ///
    /// See [`Client::with_session_token`].
    pub fn from_url_with_session_token(url: Url, session_token: impl AsRef<str>) -> Self {
        let transport = ApiTransport::new(url).with_auth(session_auth(session_token.as_ref()));

        Client {
            transport,
            env: None,
        }
    }

    #[deprecated]
    /// Please use [`Client::base_url`]
    pub fn get_endpoint(&self) -> &Url {
        self.transport.base_url()
    }

    /// The base URL every request is resolved against.
    pub fn base_url(&self) -> &Url {
        self.transport.base_url()
    }

    /// The environment this client targets, or `None` when it was built from an explicit URL.
    pub fn get_env(&self) -> Option<&Env> {
        self.env.as_ref()
    }
}

fn session_auth(session_token: &str) -> Auth {
    Auth::SessionCookie(format!("{SESSION_COOKIE}={session_token}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A base URL that carries a path prefix must keep it: the API version segment is joined
    /// onto it rather than replacing it.
    #[test]
    fn requests_preserve_the_base_url_path_prefix() {
        let client = Client::from_url_with_session_token(
            Url::parse("https://example.com/api").unwrap(),
            "token",
        );

        assert_eq!(
            client.base_url().join("v1/user").unwrap().as_str(),
            "https://example.com/api/v1/user"
        );
    }

    /// The server reads the session from the `id` cookie, so a bare token has to be named.
    #[test]
    fn session_token_is_sent_as_the_id_cookie() {
        let client = Client::with_session_token(Env::Production, "abc123");

        match client.transport.auth() {
            Auth::SessionCookie(cookie) => assert_eq!(cookie, "id=abc123"),
            auth => panic!("expected a session cookie, got {auth:?}"),
        }
    }
}
