//! Exchanging a long-lived API key for a session.

use reqwest::header::SET_COOKIE;
use serde::Serialize;

use crate::error::ClientError;
use crate::transport::{ApiTransport, Auth, ResponseExt};

/// Form body of `POST login/api-key`.
#[derive(Serialize, Clone, Debug)]
struct ApiKeyLoginRequest<'a> {
    api_key: &'a str,
}

/// Trade an API key for the session the server opens for it.
pub fn exchange_for_session(transport: &ApiTransport, api_key: &str) -> Result<Auth, ClientError> {
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
