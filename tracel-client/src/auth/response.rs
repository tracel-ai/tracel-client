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
#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSessionResponse {
    pub session_token: String,
}

/// Clamps negative values, which only a misbehaving server would send.
fn seconds_to_duration(seconds: i64) -> Duration {
    Duration::from_secs(seconds.max(0) as u64)
}
