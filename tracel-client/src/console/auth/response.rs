use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::time::Duration;

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
pub struct DeviceSessionResponse {
    pub session_token: String,
    /// Refresh token of the lineage the device authorization opened. A credential; do
    /// not display or log it. Absent from a server that predates the refresh grant.
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// Seconds left before the refresh lineage expires.
    #[serde(default)]
    pub refresh_token_expires_in: Option<i64>,
}

impl DeviceSessionResponse {
    /// Time left before the refresh lineage expires.
    pub fn refresh_token_expires_in(&self) -> Option<Duration> {
        self.refresh_token_expires_in.map(seconds_to_duration)
    }
}

/// Redacts the session token and the refresh token.
impl Debug for DeviceSessionResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceSessionResponse")
            .field("session_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("refresh_token_expires_in", &self.refresh_token_expires_in)
            .finish()
    }
}

/// Clamps negative values, which only a misbehaving server would send.
fn seconds_to_duration(seconds: i64) -> Duration {
    Duration::from_secs(seconds.max(0) as u64)
}
