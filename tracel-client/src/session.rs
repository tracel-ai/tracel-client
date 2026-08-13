//! Turning credentials into a session the client can use.
//!
//! The two kinds of [`TracelCredentials`] reach the server differently — an API
//! key has to be exchanged, a session token already is a session — and this is
//! where that difference stops mattering, so
//! [`Client::connect`](crate::Client::connect) has a single path.
//!
//! [`crate::auth`] is the other half: it issues session tokens in the first
//! place, for devices that cannot host a browser.

use reqwest::header::SET_COOKIE;
use serde::Serialize;

use crate::credentials::TracelCredentials;
use crate::error::ClientError;
use crate::transport::{ApiTransport, Auth, ResponseExt};

/// Form body of `POST login/api-key`.
#[derive(Serialize, Clone, Debug)]
struct ApiKeyLoginRequest<'a> {
    api_key: &'a str,
}

/// Establish an authenticated session on `transport`.
///
/// Only the API key path makes a request; a session token needs no exchange.
/// Neither path proves the session is live — the caller does that.
pub fn authenticate(
    transport: &ApiTransport,
    credentials: &TracelCredentials,
) -> Result<Auth, ClientError> {
    match credentials {
        TracelCredentials::ApiKey(api_key) => exchange_api_key(transport, api_key),
        TracelCredentials::SessionToken(session_token) => {
            Ok(Auth::session_token(session_token.as_str()))
        }
    }
}

/// Trade an API key for the session the server opens for it.
fn exchange_api_key(transport: &ApiTransport, api_key: &str) -> Result<Auth, ClientError> {
    let form = transport
        .request(reqwest::Method::POST, "login/api-key")
        .form(&ApiKeyLoginRequest { api_key });

    tracing::debug!("Requesting login form: {form:?}");

    let response = form.send()?.map_to_tracel_err()?;

    // The session arrives as a `Set-Cookie`, which the transport wants back
    // verbatim as the `Cookie` request header.
    let cookie = response
        .headers()
        .get(SET_COOKIE)
        .ok_or(ClientError::BadSessionId)?;

    cookie
        .to_str()
        .map(|cookie| Auth::SessionCookie(cookie.to_string()))
        .map_err(|_| ClientError::BadSessionId)
}
