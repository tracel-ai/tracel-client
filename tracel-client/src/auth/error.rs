use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

use crate::error::ClientError;

/// An OAuth 2.0 error code returned by the device authorization endpoints.
///
/// The server answers every one of these with HTTP 400 and a body of
/// `{"error": "<code>"}`.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
pub enum OAuthErrorCode {
    /// The user rejected the authorization request.
    AccessDenied,
    /// The user has not answered yet.
    AuthorizationPending,
    /// The device code is unknown, already used, or past its lifetime.
    ExpiredToken,
    /// The request was malformed or is missing a parameter.
    InvalidRequest,
    /// The client polled faster than the interval it was given.
    SlowDown,
    /// The `grant_type` is not supported by the token endpoint.
    UnsupportedGrantType,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct OAuthErrorResponse {
    pub error: OAuthErrorCode,
}

/// Errors returned by the device authorization flow.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeviceFlowError {
    /// The user rejected the authorization request. This is terminal: a new
    /// device code has to be requested.
    #[error("The device authorization request was denied")]
    AccessDenied,
    /// The device code is no longer usable, either because it outlived
    /// `expires_in` or because it was already exchanged.
    #[error("The device code expired before it was approved")]
    ExpiredToken,
    /// The device code outlived its `expires_in` while
    /// [`wait_for_approval`](super::DeviceAuthClient::wait_for_approval) was
    /// polling it.
    #[error("The device authorization was not approved within {}s", .0.as_secs())]
    TimedOut(Duration),
    /// Any other OAuth error reported by the server.
    #[error("Device authorization failed: {0}")]
    OAuth(OAuthErrorCode),
    #[error(transparent)]
    Client(#[from] ClientError),
}

impl From<OAuthErrorCode> for DeviceFlowError {
    fn from(code: OAuthErrorCode) -> Self {
        match code {
            OAuthErrorCode::AccessDenied => DeviceFlowError::AccessDenied,
            OAuthErrorCode::ExpiredToken => DeviceFlowError::ExpiredToken,
            other => DeviceFlowError::OAuth(other),
        }
    }
}
