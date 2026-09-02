use std::time::Duration;

use reqwest::Url;
use reqwest::header::COOKIE;

use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};

/// What an API call is given. These carry JSON in and JSON out, so a request
/// still running after this is a server that is not going to answer.
const API_CALL_TIMEOUT: Duration = Duration::from_secs(60);

/// What an upload is given for every megabyte it carries.
///
/// An upload is one request whose duration is set by the sender's bandwidth, so
/// no single fixed timeout serves it: short enough for a small file and it kills
/// a large one on a slow link, long enough for a large file and a stalled small
/// one hangs. This scales the allowance with what is actually being sent, at a
/// rate a link far slower than any office connection still meets.
const UPLOAD_SECONDS_ALLOWED_PER_MEGABYTE: u64 = 2;

/// What even a small upload is given, so that connecting, the TLS handshake and
/// the server's own acknowledgement are never what runs out the clock.
const SHORTEST_UPLOAD_TIMEOUT: Duration = Duration::from_secs(60);

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

/// How long an upload of `size_bytes` is allowed to take.
fn timeout_worth_allowing_an_upload_of(size_bytes: u64) -> Duration {
    let megabytes = size_bytes.div_ceil(BYTES_PER_MEGABYTE);
    let scaled = Duration::from_secs(megabytes.saturating_mul(UPLOAD_SECONDS_ALLOWED_PER_MEGABYTE));

    scaled.max(SHORTEST_UPLOAD_TIMEOUT)
}

// Which variants are live depends on the enabled features, so the transport
// itself carries them all rather than gating on them.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Auth {
    None,
    SessionCookie(String),
}

#[allow(dead_code)]
impl Auth {
    const SESSION_COOKIE_NAME: &'static str = "id";

    /// The device flow returns the bare session id, not a `Set-Cookie` header,
    /// so the cookie has to be built here.
    pub fn session_token(token: &str) -> Self {
        Auth::SessionCookie(format!("{}={token}", Self::SESSION_COOKIE_NAME))
    }
}

#[derive(Debug, Clone)]
pub struct ApiTransport {
    http_client: reqwest::blocking::Client,
    /// Kept apart from `http_client` because an upload's timeout is set per
    /// request from its size, and a client-wide one would cap it. Built once
    /// rather than per upload so that a multi-part transfer reuses the
    /// connection and its TLS session instead of handshaking for every part.
    upload_client: reqwest::blocking::Client,
    base_url: Url,
    auth: Auth,
}

#[allow(unused)]
impl ApiTransport {
    pub fn new(base_url: Url) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .timeout(API_CALL_TIMEOUT)
            .build()
            .expect("failed to build HTTP client");
        let upload_client = reqwest::blocking::Client::builder()
            .timeout(None)
            .tcp_keepalive(SHORTEST_UPLOAD_TIMEOUT)
            .build()
            .expect("failed to build HTTP upload client");
        Self {
            http_client,
            upload_client,
            base_url: with_trailing_slash(base_url),
            auth: Auth::None,
        }
    }

    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.auth = auth;
        self
    }

    pub fn set_auth(&mut self, auth: Auth) {
        self.auth = auth;
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn auth(&self) -> &Auth {
        &self.auth
    }

    pub fn request(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
    ) -> reqwest::blocking::RequestBuilder {
        let url = self.join(path.as_ref());
        let request = self
            .http_client
            .request(method, url)
            .header("X-SDK-Version", env!("CARGO_PKG_VERSION"));

        match &self.auth {
            Auth::None => request,
            Auth::SessionCookie(cookie) => request.header(COOKIE, cookie),
        }
    }

    pub fn get_json<R>(&self, path: impl AsRef<str>) -> Result<R, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::GET, path, None::<serde_json::Value>)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub fn get(&self, path: impl AsRef<str>) -> Result<(), ClientError> {
        self.req(reqwest::Method::GET, path, None::<serde_json::Value>)
            .map(|_| ())
    }

    pub fn get_optional_json<R>(&self, path: impl AsRef<str>) -> Result<Option<R>, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::GET, path, None::<serde_json::Value>)?;
        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let bytes = response.bytes()?;
        Ok(Some(serde_json::from_slice::<R>(&bytes)?))
    }

    pub fn post_json<T, R>(&self, path: impl AsRef<str>, body: Option<T>) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::POST, path, body)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub fn post<T>(&self, path: impl AsRef<str>, body: Option<T>) -> Result<(), ClientError>
    where
        T: serde::Serialize,
    {
        self.req(reqwest::Method::POST, path, body).map(|_| ())
    }

    pub fn patch_json<T, R>(&self, path: impl AsRef<str>, body: Option<T>) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::PATCH, path, body)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub fn delete(&self, path: impl AsRef<str>) -> Result<(), ClientError> {
        self.req(reqwest::Method::DELETE, path, None::<serde_json::Value>)
            .map(|_| ())
    }

    pub fn delete_json<R>(&self, path: impl AsRef<str>) -> Result<R, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::DELETE, path, None::<serde_json::Value>)?;
        let bytes = response.bytes()?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub fn req<T: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<reqwest::blocking::Response, ClientError> {
        let request = self.request(method, path);

        let request = if let Some(body) = body {
            request
                .body(serde_json::to_vec(&body)?)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
        } else {
            request
        };

        tracing::debug!("Sending request to Burn API: {:?}", request);
        let response = request.send()?.map_to_tracel_err()?;
        tracing::debug!("Received response from Burn API: {:?}", response);

        Ok(response)
    }

    /// Upload raw bytes to an absolute (presigned) URL via PUT.
    ///
    /// Unlike the other helpers this does NOT join the path with `base_url` and
    /// does NOT attach auth — presigned URLs (e.g. S3) are absolute and
    /// self-authenticating.
    ///
    /// The request is given [a timeout drawn from its own
    /// size](timeout_worth_allowing_an_upload_of) rather than the one API calls
    /// get, which no upload larger than a few megabytes would survive.
    pub fn upload_bytes_to_url(&self, url: &str, bytes: Vec<u8>) -> Result<(), ClientError> {
        let timeout = timeout_worth_allowing_an_upload_of(bytes.len() as u64);

        self.upload_client
            .put(url)
            .timeout(timeout)
            .body(bytes)
            .send()?
            .map_to_tracel_err()?;

        Ok(())
    }

    pub fn join(&self, path: &str) -> Url {
        self.join_versioned(path, 1)
    }

    fn join_versioned(&self, path: &str, version: u8) -> Url {
        self.base_url
            .join(&format!("v{version}/"))
            .unwrap()
            .join(path)
            .expect("Should be able to join url")
    }
}

fn with_trailing_slash(mut base_url: Url) -> Url {
    if !base_url.path().ends_with('/') {
        let path = format!("{}/", base_url.path());
        base_url.set_path(&path);
    }
    base_url
}

pub(crate) trait ResponseExt {
    fn map_to_tracel_err(self) -> Result<reqwest::blocking::Response, ClientError>;
}

impl ResponseExt for reqwest::blocking::Response {
    fn map_to_tracel_err(self) -> Result<reqwest::blocking::Response, ClientError> {
        if self.status().is_success() {
            Ok(self)
        } else {
            match self.status() {
                reqwest::StatusCode::NOT_FOUND => {
                    let code = self
                        .text()
                        .ok()
                        .and_then(|text| text.parse::<serde_json::Value>().ok())
                        .and_then(|value| serde_json::from_value::<ApiErrorBody>(value).ok())
                        .map(|body| body.code);

                    match code {
                        Some(code) => Err(ClientError::NotFoundWithCode(code)),
                        None => Err(ClientError::NotFound),
                    }
                }
                reqwest::StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
                reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(ClientError::InternalServerError),
                _ => Err(ClientError::ApiError {
                    status: self.status(),
                    body: self
                        .text()
                        .map_err(|e| ClientError::UnknownError(e.to_string()))?
                        .parse::<serde_json::Value>()
                        .and_then(serde_json::from_value::<ApiErrorBody>)
                        .unwrap_or_else(|e| ApiErrorBody {
                            code: ApiErrorCode::Unknown,
                            message: e.to_string(),
                        }),
                }),
            }
        }
    }
}
