use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::time::Duration;

use crate::console::{AccessToken, RefreshToken};

/// A pending device authorization (RFC 8628 §3.2).
#[derive(Deserialize, Serialize, Clone)]
pub struct DeviceCodeResponse {
    /// Code the client polls with. A credential; do not display or log it.
    pub device_code: String,
    /// Code the user enters on the verification page, e.g. `BCDF-GHJK`.
    pub user_code: String,
    /// Page the user opens to approve the request.
    pub verification_uri: String,
    /// [`Self::verification_uri`] with the user code filled in.
    pub verification_uri_complete: String,
    /// Lifetime of the codes, in seconds.
    pub expires_in: i64,
    /// Minimum seconds between two polls.
    pub interval: i64,
}

impl DeviceCodeResponse {
    /// Lifetime of the codes.
    pub fn expires_in(&self) -> Duration {
        seconds_to_duration(self.expires_in)
    }

    /// Minimum delay between two polls.
    pub fn interval(&self) -> Duration {
        seconds_to_duration(self.interval)
    }
}

/// Redacts the device code.
impl Debug for DeviceCodeResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceCodeResponse")
            .field("device_code", &"[REDACTED]")
            .field("user_code", &self.user_code)
            .field("verification_uri", &self.verification_uri)
            .field("verification_uri_complete", &self.verification_uri_complete)
            .field("expires_in", &self.expires_in)
            .field("interval", &self.interval)
            .finish()
    }
}

/// Body of a successful `POST auth/token`.
#[derive(Deserialize, Clone)]
pub struct AppTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_token_expires_in: i64,
}

/// Redacts the access token and the refresh token.
impl Debug for AppTokenResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppTokenResponse")
            .field("access_token", &"[REDACTED]")
            .field("expires_in", &self.expires_in)
            .field("refresh_token", &"[REDACTED]")
            .field("refresh_token_expires_in", &self.refresh_token_expires_in)
            .finish()
    }
}

impl From<AppTokenResponse> for IssuedAppSession {
    fn from(response: AppTokenResponse) -> Self {
        Self {
            access_token: AccessToken::new(response.access_token),
            access_token_expires_in: seconds_to_duration(response.expires_in),
            refresh_token: RefreshToken::new(response.refresh_token),
            refresh_token_expires_in: seconds_to_duration(response.refresh_token_expires_in),
        }
    }
}

/// What the token endpoint grants: an app session's access token and the
/// refresh token that renews it.
#[derive(Debug, Clone)]
pub struct IssuedAppSession {
    /// Token to connect a [`Client`](crate::console::Client) with.
    pub access_token: AccessToken,
    /// Time left before the access token expires, an hour at most.
    pub access_token_expires_in: Duration,
    /// Token to spend on the next [`DeviceAuthClient::refresh`]. Every
    /// refresh rotates it, so the one that comes back replaces the one spent.
    ///
    /// [`DeviceAuthClient::refresh`]: super::DeviceAuthClient::refresh
    pub refresh_token: RefreshToken,
    /// Time left before the app session ends and the user has to sign in
    /// again. Set when the device authorization was approved; refreshing does
    /// not extend it.
    pub refresh_token_expires_in: Duration,
}

/// Clamps negative values, which only a misbehaving server would send.
fn seconds_to_duration(seconds: i64) -> Duration {
    Duration::from_secs(seconds.max(0) as u64)
}
