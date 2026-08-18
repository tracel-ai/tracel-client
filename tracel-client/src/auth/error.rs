use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

use crate::error::ClientError;

/// OAuth 2.0 error code returned by the device authorization endpoints.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, strum::Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
pub enum OAuthErrorCode {
    /// The user rejected the request.
    AccessDenied,
    /// The user has not answered yet.
    AuthorizationPending,
    /// The device code is unknown, spent, or past its lifetime.
    ExpiredToken,
    /// The request was malformed or incomplete.
    InvalidRequest,
    /// The client polled faster than its interval.
    SlowDown,
    /// The token endpoint does not support the `grant_type`.
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
    /// The user rejected the request. Terminal: request a new device code.
    #[error("The device authorization request was denied")]
    AccessDenied,
    /// The device code is spent or past its lifetime.
    #[error("The device code expired before it was approved")]
    ExpiredToken,
    /// The code expired while [`wait_for_approval`] was polling it.
    ///
    /// [`wait_for_approval`]: super::DeviceAuthClient::wait_for_approval
    #[error("The device authorization was not approved within {}s", .0.as_secs())]
    TimedOut(Duration),
    /// Any other error reported by the server.
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
