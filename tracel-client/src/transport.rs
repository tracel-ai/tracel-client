use std::time::Duration;

use reqwest::Url;
use reqwest::header::{AUTHORIZATION, HeaderValue};

#[cfg(feature = "console")]
use crate::console::{app_session::AppSession, credentials::AccessToken};
use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};

const API_CALL_TIMEOUT: Duration = Duration::from_secs(120);

const UPLOAD_SECONDS_ALLOWED_PER_MEGABYTE: u64 = 10;

const MIN_UPLOAD_TIMEOUT: Duration = Duration::from_secs(120);

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

fn timeout_worth_allowing_an_upload_of(size_bytes: u64) -> Duration {
    const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;
    let megabytes = size_bytes.div_ceil(BYTES_PER_MEGABYTE);
    let allowed =
        Duration::from_secs(megabytes.saturating_mul(UPLOAD_SECONDS_ALLOWED_PER_MEGABYTE));

    allowed.max(MIN_UPLOAD_TIMEOUT)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Auth {
    None,
    Bearer(HeaderValue),
    #[cfg(feature = "console")]
    AppSession(AppSession),
}

/// What one request carries, resolved from its [`Auth`] when it leaves.
pub(crate) enum Presented {
    Nothing,
    Bearer(HeaderValue),
    #[cfg(feature = "console")]
    AppSessionAccess(AccessToken, HeaderValue),
}

impl Presented {
    pub(crate) fn header(&self) -> Option<&HeaderValue> {
        match self {
            Presented::Nothing => None,
            Presented::Bearer(value) => Some(value),
            #[cfg(feature = "console")]
            Presented::AppSessionAccess(_, value) => Some(value),
        }
    }
}

#[allow(dead_code)]
impl Auth {
    /// An `Authorization: Bearer` credential, marked sensitive so that logging
    /// a request never prints it.
    pub fn bearer(token: &str) -> Result<Self, ClientError> {
        bearer_header(token).map(Auth::Bearer)
    }

    /// The credential a request leaving now carries. An app session renews
    /// its access token first when it is about to expire.
    pub(crate) fn present(&self) -> Result<Presented, ClientError> {
        match self {
            Auth::None => Ok(Presented::Nothing),
            Auth::Bearer(value) => Ok(Presented::Bearer(value.clone())),
            #[cfg(feature = "console")]
            Auth::AppSession(session) => {
                let access_token = session.access_token()?;
                let header = bearer_header(access_token.as_str())?;
                Ok(Presented::AppSessionAccess(access_token, header))
            }
        }
    }

    /// What to carry instead of `refused`, which the server answered with 401,
    /// or `None` when this credential cannot be renewed.
    pub(crate) fn present_after_refusal(
        &self,
        refused: &Presented,
    ) -> Result<Option<Presented>, ClientError> {
        match (self, refused) {
            #[cfg(feature = "console")]
            (Auth::AppSession(session), Presented::AppSessionAccess(rejected, _)) => {
                let access_token = session.access_token_after_rejection(rejected)?;
                let header = bearer_header(access_token.as_str())?;
                Ok(Some(Presented::AppSessionAccess(access_token, header)))
            }
            _ => Ok(None),
        }
    }
}

fn bearer_header(token: &str) -> Result<HeaderValue, ClientError> {
    let mut value = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|_| ClientError::Unauthenticated)?;
    value.set_sensitive(true);
    Ok(value)
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
            .tcp_keepalive(MIN_UPLOAD_TIMEOUT)
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
    ) -> Result<reqwest::blocking::RequestBuilder, ClientError> {
        let presented = self.auth.present()?;
        Ok(self.request_carrying(method, self.join(path.as_ref()), &presented))
    }

    fn request_carrying(
        &self,
        method: reqwest::Method,
        url: Url,
        presented: &Presented,
    ) -> reqwest::blocking::RequestBuilder {
        let request = self
            .http_client
            .request(method, url)
            .header("X-SDK-Version", env!("CARGO_PKG_VERSION"));

        match presented.header() {
            Some(value) => request.header(AUTHORIZATION, value.clone()),
            None => request,
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
        let body = body.map(|body| serde_json::to_vec(&body)).transpose()?;
        let url = self.join(path.as_ref());

        let presented = self.auth.present()?;
        let response = self.send(method.clone(), url.clone(), &presented, body.clone())?;
        if response.status() != reqwest::StatusCode::UNAUTHORIZED {
            return response.map_to_tracel_err();
        }

        match self.auth.present_after_refusal(&presented)? {
            Some(renewed) => self.send(method, url, &renewed, body)?.map_to_tracel_err(),
            None => response.map_to_tracel_err(),
        }
    }

    fn send(
        &self,
        method: reqwest::Method,
        url: Url,
        presented: &Presented,
        body: Option<Vec<u8>>,
    ) -> Result<reqwest::blocking::Response, ClientError> {
        let request = self.request_carrying(method, url, presented);
        let request = match body {
            Some(body) => request
                .body(body)
                .header(reqwest::header::CONTENT_TYPE, "application/json"),
            None => request,
        };

        tracing::debug!("Sending request to Burn API: {:?}", request);
        let response = request.send()?;
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
                reqwest::StatusCode::UNAUTHORIZED => Err(ClientError::Unauthenticated),
                reqwest::StatusCode::INTERNAL_SERVER_ERROR => Err(ClientError::InternalServerError),
                status => {
                    let body = self
                        .text()
                        .map_err(|e| ClientError::UnknownError(e.to_string()))?
                        .parse::<serde_json::Value>()
                        .and_then(serde_json::from_value::<ApiErrorBody>)
                        .unwrap_or_else(|e| ApiErrorBody {
                            code: ApiErrorCode::Unknown,
                            message: e.to_string(),
                        });
                    Err(match body.code {
                        ApiErrorCode::CredentialNotAllowed => ClientError::CredentialNotAllowed,
                        _ => ClientError::ApiError { status, body },
                    })
                }
            }
        }
    }
}
