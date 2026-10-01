use std::fmt::{Debug, Formatter};

use crate::console::app_session::AppSession;

/// Credentials to connect to the Tracel server.
///
/// Every kind authorizes a [`Client`](crate::console::Client) through
/// [`Client::connect`](crate::console::Client::connect), sent as an
/// `Authorization: Bearer` token.
#[derive(Clone)]
pub enum TracelCredentials {
    /// An API key created from the Tracel console. It acts on the projects of
    /// the one namespace it was created for.
    ApiKey(String),
    /// The access token of an app session whose renewal belongs to someone
    /// else, the application that holds its refresh token. It lasts an hour.
    AccessToken(AccessToken),
    /// An app session the client renews by itself through its
    /// [`SessionStore`](crate::console::app_session::SessionStore).
    AppSession(AppSession),
}

impl TracelCredentials {
    /// Credentials backed by an API key.
    pub fn api_key(api_key: impl Into<String>) -> Self {
        Self::ApiKey(api_key.into())
    }

    /// Credentials backed by an access token the caller keeps renewed.
    pub fn access_token(access_token: AccessToken) -> Self {
        Self::AccessToken(access_token)
    }

    /// Credentials backed by an app session the client renews.
    pub fn app_session(app_session: AppSession) -> Self {
        Self::AppSession(app_session)
    }

    /// Reads an API key from `TRACEL_API_KEY`; an empty value reads as unset.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        match std::env::var("TRACEL_API_KEY") {
            Ok(api_key) if !api_key.is_empty() => Ok(Self::ApiKey(api_key)),
            Ok(_) => Err(std::env::VarError::NotPresent),
            Err(error) => Err(error),
        }
    }
}

/// Redacts the secret.
impl Debug for TracelCredentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey(_) => f.write_str("TracelCredentials::ApiKey([REDACTED])"),
            Self::AccessToken(_) => f.write_str("TracelCredentials::AccessToken([REDACTED])"),
            Self::AppSession(_) => f.write_str("TracelCredentials::AppSession([REDACTED])"),
        }
    }
}

impl From<AccessToken> for TracelCredentials {
    fn from(access_token: AccessToken) -> Self {
        Self::AccessToken(access_token)
    }
}

impl From<AppSession> for TracelCredentials {
    fn from(app_session: AppSession) -> Self {
        Self::AppSession(app_session)
    }
}

/// The access token of an app session, `tcl_at_...`.
///
/// Issued with a [`RefreshToken`] by
/// [`DeviceAuthClient`](crate::console::auth::DeviceAuthClient) and valid for
/// an hour.
#[derive(Clone, PartialEq, Eq)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

/// Redacts the token.
impl Debug for AccessToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccessToken([REDACTED])")
    }
}

/// The refresh token of an app session, `tcl_rt_...`.
///
/// Every refresh spends it and returns its successor, so the one that comes
/// back replaces the one that was spent. The app session it belongs to ends
/// seven days after the device authorization that opened it, however often it
/// is refreshed.
#[derive(Clone, PartialEq, Eq)]
pub struct RefreshToken(String);

impl RefreshToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

/// Redacts the token.
impl Debug for RefreshToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("RefreshToken([REDACTED])")
    }
}
