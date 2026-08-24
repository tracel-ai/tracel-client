//! Turning [`TracelCredentials`] into a session.
//!
//! An API key has to be exchanged for one; a session token already is one.

use reqwest::header::SET_COOKIE;
use serde::Serialize;

use crate::console::credentials::TracelCredentials;
use crate::error::ClientError;
use crate::transport::{ApiTransport, Auth, ResponseExt};

/// Form body of `POST login/api-key`.
#[derive(Serialize, Clone, Debug)]
struct ApiKeyLoginRequest<'a> {
    api_key: &'a str,
}

/// Establishes an authenticated session on `transport`.
///
/// Neither path proves the session is live; the caller does that.
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

/// Trades an API key for the session the server opens for it.
fn exchange_api_key(transport: &ApiTransport, api_key: &str) -> Result<Auth, ClientError> {
    let form = transport
        .request(reqwest::Method::POST, "login/api-key")
        .form(&ApiKeyLoginRequest { api_key });

    tracing::debug!("Requesting login form: {form:?}");

    let response = form.send()?.map_to_tracel_err()?;

    // The transport sends this back verbatim as the `Cookie` request header.
    let cookie = response
        .headers()
        .get(SET_COOKIE)
        .ok_or(ClientError::BadSessionId)?;

    cookie
        .to_str()
        .map(|cookie| Auth::SessionCookie(cookie.to_string()))
        .map_err(|_| ClientError::BadSessionId)
}
