use std::fmt::{Debug, Formatter};
use std::str::FromStr;

/// Credentials to connect to the Tracel server.
///
/// Either kind authorizes a [`Client`](crate::console::Client) through
/// [`Client::connect`](crate::console::Client::connect).
#[derive(Clone, PartialEq, Eq)]
pub enum TracelCredentials {
    /// A long-lived API key, created from the Tracel console.
    ApiKey(String),
    /// A session token, issued by the device authorization flow.
    SessionToken(SessionToken),
}

impl TracelCredentials {
    /// Credentials backed by an API key.
    pub fn api_key(api_key: impl Into<String>) -> Self {
        Self::ApiKey(api_key.into())
    }

    /// Credentials backed by a session token.
    pub fn session_token(session_token: SessionToken) -> Self {
        Self::SessionToken(session_token)
    }

    /// Reads credentials from the environment.
    ///
    /// `TRACEL_API_KEY` takes precedence over `TRACEL_SESSION_TOKEN`.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        if let Ok(api_key) = std::env::var("TRACEL_API_KEY") {
            return Ok(Self::ApiKey(api_key));
        }

        SessionToken::from_env().map(Self::SessionToken)
    }
}

/// Redacts the secret.
impl Debug for TracelCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey(_) => f.write_str("TracelCredentials::ApiKey([REDACTED])"),
            Self::SessionToken(_) => f.write_str("TracelCredentials::SessionToken([REDACTED])"),
        }
    }
}

impl From<SessionToken> for TracelCredentials {
    fn from(session_token: SessionToken) -> Self {
        Self::SessionToken(session_token)
    }
}

/// An opaque Tracel session token.
///
/// Issued by [`DeviceAuthClient`](crate::console::auth::DeviceAuthClient). Expires after
/// a day of inactivity.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken(String);

impl SessionToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// Reads a session token from `TRACEL_SESSION_TOKEN`.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let token = std::env::var("TRACEL_SESSION_TOKEN")?;
        Ok(Self::new(token))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

/// Redacts the token.
impl Debug for SessionToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionToken([REDACTED])")
    }
}

impl FromStr for SessionToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("Session token cannot be empty".to_string())
        } else {
            Ok(Self::new(s))
        }
    }
}
