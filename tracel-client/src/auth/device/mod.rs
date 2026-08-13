//! OAuth 2.0 Device Authorization Grant (RFC 8628).
//!
//! Lets a device that cannot host a browser — a CLI, a training node, a
//! headless station — obtain a Burn Central session by asking the user to
//! approve a short code on another device.
//!
//! ```no_run
//! use tracel_client::{Client, DeviceAuthClient, Env, TracelCredentials};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let device_auth = DeviceAuthClient::new(Env::Production, "tracel-cli");
//!
//! let session_token = device_auth.authorize(|authorization| {
//!     println!("Open {}", authorization.verification_uri_complete);
//!     println!("and confirm the code {}", authorization.user_code);
//! })?;
//!
//! // The session token can be persisted and reused until it expires.
//! let credentials = TracelCredentials::session_token(session_token);
//! let client = Client::connect(Env::Production, &credentials)?;
//! println!("Signed in as {}", client.user().username);
//! # Ok(())
//! # }
//! ```
//!
//! [`DeviceAuthClient::authorize`] blocks until the user answers. Drive the
//! flow yourself with [`DeviceAuthClient::start`] and
//! [`DeviceAuthClient::poll`] when that does not fit.

mod request;

pub mod error;
pub mod response;

use std::time::{Duration, Instant};

use reqwest::Url;

use crate::client::Env;
use crate::credentials::SessionToken;
use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};
use crate::transport::{ApiTransport, ResponseExt};

use error::OAuthErrorResponse;
use request::{DeviceCodeRequest, DeviceTokenRequest};
use response::DeviceSessionResponse;

pub use error::{DeviceFlowError, OAuthErrorCode};
pub use response::DeviceCodeResponse;

/// Grant type identifier of the device authorization flow (RFC 8628 section 3.4).
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// Added to the poll interval every time the server answers `slow_down`.
///
/// The server permanently raises its own required interval by this much and
/// does not report the new value, so the client has to mirror the increase.
const SLOW_DOWN_INCREMENT: Duration = Duration::from_secs(5);

/// Floor for the poll interval, in case the server reports a zero interval.
const MIN_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Result of a single poll of the token endpoint.
#[derive(Debug, Clone)]
pub enum DevicePollOutcome {
    /// The user has not answered yet. Poll again after the interval.
    Pending,
    /// The last poll came too early. Add five seconds to the interval before
    /// polling again — the server raised its own requirement by that much.
    SlowDown,
    /// The user approved the request.
    Approved(SessionToken),
}

/// A client for the OAuth 2.0 Device Authorization Grant.
///
/// Unauthenticated by construction — obtaining credentials is what it is for.
#[derive(Debug, Clone)]
pub struct DeviceAuthClient {
    transport: ApiTransport,
    client_id: String,
}

impl DeviceAuthClient {
    /// Create a client for the given environment.
    ///
    /// `client_id` identifies the application requesting authorization, for
    /// example `tracel-cli`. It is echoed back on every poll and must be
    /// non-empty, at most 128 bytes, free of control characters, and free of
    /// leading and trailing whitespace.
    pub fn new(env: Env, client_id: impl Into<String>) -> Self {
        Self {
            transport: ApiTransport::new(env.get_url()),
            client_id: client_id.into(),
        }
    }

    /// Create a client with a custom base URL.
    pub fn from_url(url: Url, client_id: impl Into<String>) -> Self {
        Self {
            transport: ApiTransport::new(url),
            client_id: client_id.into(),
        }
    }

    /// The client identifier this client authorizes as.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Run the whole flow: request a device code, hand it to `on_started` so
    /// it can be shown to the user, then block until the request is answered.
    pub fn authorize<F>(&self, on_started: F) -> Result<SessionToken, DeviceFlowError>
    where
        F: FnOnce(&DeviceCodeResponse),
    {
        let authorization = self.start()?;
        on_started(&authorization);
        self.wait_for_approval(&authorization)
    }

    /// Request a device code.
    ///
    /// The returned [`DeviceCodeResponse`] carries the user code and the
    /// verification URI to show the user, and the device code to poll with.
    pub fn start(&self) -> Result<DeviceCodeResponse, DeviceFlowError> {
        self.post_form(
            "auth/device/code",
            &DeviceCodeRequest {
                client_id: &self.client_id,
            },
        )
    }

    /// Poll the token endpoint once.
    ///
    /// Returns an error only when the flow cannot continue — the user denied
    /// the request, the code expired, or the request itself was rejected.
    /// A user who has simply not answered yet yields
    /// [`DevicePollOutcome::Pending`].
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

    /// Poll until the user answers, the code expires, or the request is denied.
    ///
    /// Blocks the calling thread, waiting the interval the server asked for
    /// between polls and backing off further whenever it answers `slow_down`.
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

            // Wait before the first poll as well: the answer can only be
            // `pending` until the user has had time to open the page, and
            // polling too eagerly makes the server raise the interval.
            std::thread::sleep(interval.min(remaining));

            match self.poll(&authorization.device_code)? {
                DevicePollOutcome::Approved(token) => return Ok(token),
                DevicePollOutcome::Pending => {}
                DevicePollOutcome::SlowDown => interval += SLOW_DOWN_INCREMENT,
            }
        }
    }

    /// Send a form-encoded request and decode a JSON response.
    ///
    /// The device authorization endpoints are form-encoded rather than JSON,
    /// and report failures as `{"error": "<code>"}` with HTTP 400 instead of
    /// the usual `{"code", "message"}` envelope, so neither
    /// [`ApiTransport::post_json`] nor the generic error mapping applies.
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

/// Turn an HTTP 400 response into a device flow error.
fn bad_request_error(response: reqwest::blocking::Response) -> DeviceFlowError {
    let status = response.status();
    match response.text() {
        Ok(body) => parse_bad_request_body(status, body),
        Err(error) => ClientError::from(error).into(),
    }
}

/// These endpoints answer 400 with an OAuth envelope for flow errors, but with
/// the usual `{"code", "message"}` envelope when the request is rejected before
/// the flow is entered — an invalid `client_id`, for instance.
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
