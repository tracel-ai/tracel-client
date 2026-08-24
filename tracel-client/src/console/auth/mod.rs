//! OAuth 2.0 Device Authorization Grant ([RFC 8628]).
//!
//! Obtains a session on a device that cannot host a browser, by having the user
//! approve a short code elsewhere.
//!
//! # Examples
//!
//! ```no_run
//! use tracel_client::console::{Client, Env, TracelCredentials, auth::DeviceAuthClient};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let device_auth = DeviceAuthClient::new(Env::Production, "tracel-cli");
//!
//! let token = device_auth.authorize(|auth| {
//!     println!("Open {} and enter {}", auth.verification_uri, auth.user_code);
//! })?;
//!
//! let client = Client::connect(Env::Production, &TracelCredentials::session_token(token))?;
//! # Ok(())
//! # }
//! ```
//!
//! [RFC 8628]: https://datatracker.ietf.org/doc/html/rfc8628

mod error;
mod request;
mod response;

use std::time::{Duration, Instant};

use reqwest::Url;

use crate::console::client::Env;
use crate::console::credentials::SessionToken;
use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};
use crate::transport::{ApiTransport, ResponseExt};

use error::OAuthErrorResponse;
use request::{DeviceCodeRequest, DeviceTokenRequest};
use response::DeviceSessionResponse;

pub use error::{DeviceFlowError, OAuthErrorCode};
pub use response::DeviceCodeResponse;

/// `grant_type` of the device authorization flow (RFC 8628 §3.4).
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// Added to the poll interval on every `slow_down`.
///
/// The server raises its own interval by this much without reporting the new
/// value, so the client mirrors it.
const SLOW_DOWN_INCREMENT: Duration = Duration::from_secs(5);

/// Lower bound on the poll interval.
const MIN_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Outcome of a single [`DeviceAuthClient::poll`].
#[derive(Debug, Clone)]
pub enum DevicePollOutcome {
    /// The user has not answered yet.
    Pending,
    /// Polled too soon; back off by five seconds.
    SlowDown,
    /// The user approved the request.
    Approved(SessionToken),
}

/// Client for the device authorization flow.
#[derive(Debug, Clone)]
pub struct DeviceAuthClient {
    transport: ApiTransport,
    client_id: String,
}

impl DeviceAuthClient {
    /// Creates a client for `env`.
    ///
    /// `client_id` identifies the requesting application, e.g. `tracel-cli`. It
    /// must be non-empty, at most 128 bytes, and free of control characters and
    /// surrounding whitespace.
    pub fn new(env: Env, client_id: impl Into<String>) -> Self {
        Self {
            transport: ApiTransport::new(env.get_url()),
            client_id: client_id.into(),
        }
    }

    /// Creates a client for a custom base URL.
    pub fn from_url(url: Url, client_id: impl Into<String>) -> Self {
        Self {
            transport: ApiTransport::new(url),
            client_id: client_id.into(),
        }
    }

    /// Returns the client identifier.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Runs the flow to completion.
    ///
    /// Calls `on_started` with the pending authorization so it can be shown to
    /// the user, then blocks until the request is answered. Use [`start`] and
    /// [`poll`] to drive the flow manually.
    ///
    /// [`start`]: Self::start
    /// [`poll`]: Self::poll
    pub fn authorize<F>(&self, on_started: F) -> Result<SessionToken, DeviceFlowError>
    where
        F: FnOnce(&DeviceCodeResponse),
    {
        let authorization = self.start()?;
        on_started(&authorization);
        self.wait_for_approval(&authorization)
    }

    /// Requests a device code.
    pub fn start(&self) -> Result<DeviceCodeResponse, DeviceFlowError> {
        self.post_form(
            "auth/device/code",
            &DeviceCodeRequest {
                client_id: &self.client_id,
            },
        )
    }

    /// Polls the token endpoint once.
    ///
    /// A pending or throttled poll is an [`Ok`] outcome; only a terminal
    /// failure is an error.
    pub fn poll(&self, device_code: &str) -> Result<DevicePollOutcome, DeviceFlowError> {
        let request = DeviceTokenRequest {
            grant_type: DEVICE_CODE_GRANT,
            device_code,
            client_id: &self.client_id,
        };

        match self.post_form::<_, DeviceSessionResponse>("auth/token", &request) {
            Ok(response) => Ok(DevicePollOutcome::Approved(SessionToken::new(
                response.session_token,
            ))),
            Err(DeviceFlowError::OAuth(OAuthErrorCode::AuthorizationPending)) => {
                Ok(DevicePollOutcome::Pending)
            }
            Err(DeviceFlowError::OAuth(OAuthErrorCode::SlowDown)) => {
                Ok(DevicePollOutcome::SlowDown)
            }
            Err(error) => Err(error),
        }
    }

    /// Blocks until the user answers or `authorization` expires.
    ///
    /// Sleeps for the interval the server asked for between polls, backing off
    /// further on `slow_down`.
    pub fn wait_for_approval(
        &self,
        authorization: &DeviceCodeResponse,
    ) -> Result<SessionToken, DeviceFlowError> {
        let lifetime = authorization.expires_in();
        let deadline = Instant::now() + lifetime;
        let mut interval = authorization.interval().max(MIN_POLL_INTERVAL);

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(DeviceFlowError::TimedOut(lifetime));
            }

            // Sleep before the first poll too: polling eagerly only makes the
            // server raise the interval.
            std::thread::sleep(interval.min(remaining));

            match self.poll(&authorization.device_code)? {
                DevicePollOutcome::Approved(token) => return Ok(token),
                DevicePollOutcome::Pending => {}
                DevicePollOutcome::SlowDown => interval += SLOW_DOWN_INCREMENT,
            }
        }
    }

    /// Sends a form-encoded request and decodes a JSON response.
    ///
    /// These endpoints take forms rather than JSON and report failures as
    /// `{"error": "<code>"}`, so the transport's JSON helpers do not apply.
    fn post_form<B, R>(&self, path: &str, body: &B) -> Result<R, DeviceFlowError>
    where
        B: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let request = self
            .transport
            .request(reqwest::Method::POST, path)
            .form(body);

        tracing::debug!("Sending device authorization request to Burn API: {request:?}");
        let response = request.send().map_err(ClientError::from)?;
        tracing::debug!("Received device authorization response from Burn API: {response:?}");

        if response.status() == reqwest::StatusCode::BAD_REQUEST {
            return Err(bad_request_error(response));
        }

        let response = response.map_to_tracel_err()?;
        let bytes = response.bytes().map_err(ClientError::from)?;
        serde_json::from_slice(&bytes).map_err(|error| ClientError::from(error).into())
    }
}

/// Maps an HTTP 400 response to a [`DeviceFlowError`].
fn bad_request_error(response: reqwest::blocking::Response) -> DeviceFlowError {
    let status = response.status();
    match response.text() {
        Ok(body) => parse_bad_request_body(status, body),
        Err(error) => ClientError::from(error).into(),
    }
}

/// Flow errors use the OAuth envelope; requests rejected before the flow starts,
/// such as an invalid `client_id`, use the usual `{"code", "message"}` one.
fn parse_bad_request_body(status: reqwest::StatusCode, body: String) -> DeviceFlowError {
    if let Ok(oauth) = serde_json::from_str::<OAuthErrorResponse>(&body) {
        return oauth.error.into();
    }

    let body = serde_json::from_str::<ApiErrorBody>(&body).unwrap_or(ApiErrorBody {
        code: ApiErrorCode::Unknown,
        message: body,
    });

    ClientError::ApiError { status, body }.into()
}
