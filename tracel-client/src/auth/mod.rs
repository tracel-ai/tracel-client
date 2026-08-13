//! Turning credentials into a session the client can use.
//!
//! The two kinds of [`TracelCredentials`] reach the server differently — an API
//! key has to be exchanged, a session token already is a session — and this is
//! where that difference stops mattering, so
//! [`Client::connect`](crate::Client::connect) has a single path.
//!
//! [`device`] is the other half: it issues session tokens in the first place,
//! for devices that cannot host a browser.

mod api_key;

pub mod device;

pub use device::{
    DeviceAuthClient, DeviceCodeResponse, DeviceFlowError, DevicePollOutcome, OAuthErrorCode,
};

use crate::credentials::TracelCredentials;
use crate::error::ClientError;
use crate::transport::{ApiTransport, Auth};

/// Establish an authenticated session on `transport`.
///
/// Only the API key path makes a request; a session token needs no exchange.
/// Neither path proves the session is live — the caller does that.
pub fn authenticate(
    transport: &ApiTransport,
    credentials: &TracelCredentials,
) -> Result<Auth, ClientError> {
    match credentials {
        TracelCredentials::ApiKey(api_key) => api_key::exchange_for_session(transport, api_key),
        TracelCredentials::SessionToken(session_token) => {
            Ok(Auth::session_token(session_token.as_str()))
        }
    }
}
