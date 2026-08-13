use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::time::Duration;

/// A pending device authorization, as returned by `POST auth/device/code`.
///
/// Mirrors the OAuth 2.0 device authorization response (RFC 8628 section 3.2).
#[derive(Deserialize, Serialize, Clone)]
pub struct DeviceCodeResponse {
    /// Opaque code the client polls the token endpoint with. This is a
    /// credential — never display it or write it to a log.
    pub device_code: String,
    /// Short hyphenated code the user types on the verification page,
    /// for example `BCDF-GHJK`.
    pub user_code: String,
    /// Page the user has to open to approve the request.
    pub verification_uri: String,
    /// [`Self::verification_uri`] with the user code already filled in.
    pub verification_uri_complete: String,
    /// Lifetime of the device code and user code, in seconds.
    pub expires_in: i64,
    /// Minimum number of seconds the client must wait between two polls.
    pub interval: i64,
}

impl DeviceCodeResponse {
    /// Lifetime of the device code.
    pub fn expires_in(&self) -> Duration {
        seconds_to_duration(self.expires_in)
    }

    /// Minimum delay between two polls of the token endpoint.
    pub fn interval(&self) -> Duration {
        seconds_to_duration(self.interval)
    }
}

/// Redacts the device code, which is a bearer credential for the whole flow.
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

/// Body of a successful `POST auth/token` exchange.
#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSessionResponse {
    pub session_token: String,
}

/// Negative durations are not representable and only ever come from a
/// misbehaving server, so they are clamped to zero.
fn seconds_to_duration(seconds: i64) -> Duration {
    Duration::from_secs(seconds.max(0) as u64)
}
