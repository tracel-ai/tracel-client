use std::time::Duration;

use reqwest::Url;
use reqwest::header::COOKIE;

use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};

/// What an API call is given. These carry JSON in and JSON out, so a request
/// still running after this is a server that is not going to answer.
const API_CALL_TIMEOUT: Duration = Duration::from_secs(60);

/// Deliberately generous: a total timeout cannot tell a slow upload from a
/// stalled one, and [`CONNECT_TIMEOUT`] plus TCP keepalive already catch a host
/// that is not there. Tightened towards real bandwidth it would start failing
/// uploads that would have succeeded, which is what it exists to prevent.
const UPLOAD_SECONDS_ALLOWED_PER_MEGABYTE: u64 = 10;

/// What even a small upload is given, so that connecting, the TLS handshake and
/// the server's own acknowledgement are never what runs out the clock.
const SHORTEST_UPLOAD_TIMEOUT: Duration = Duration::from_secs(60);

/// An unreachable server fails here in seconds rather than sitting out the whole
/// transfer allowance.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

fn timeout_worth_allowing_an_upload_of(size_bytes: u64) -> Duration {
    let megabytes = size_bytes.div_ceil(BYTES_PER_MEGABYTE);
    let allowed =
        Duration::from_secs(megabytes.saturating_mul(UPLOAD_SECONDS_ALLOWED_PER_MEGABYTE));

    allowed.max(SHORTEST_UPLOAD_TIMEOUT)
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
            .connect_timeout(CONNECT_TIMEOUT)
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
