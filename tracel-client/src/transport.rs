use std::time::Duration;

use reqwest::Url;
use reqwest::header::COOKIE;

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

/// HTTP transport shared by every endpoint.
///
/// Requests are resolved against the base URL, carry the session, and time out
/// after [`API_CALL_TIMEOUT`]. Presigned uploads go through a separate client
/// and draw their timeout from their size instead.
#[derive(Debug, Clone)]
pub struct ApiTransport {
    http_client: reqwest::Client,
    upload_client: reqwest::Client,
    base_url: Url,
    auth: Auth,
}

#[allow(unused)]
impl ApiTransport {
    pub fn new(base_url: Url) -> Self {
        let upload_client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .tcp_keepalive(MIN_UPLOAD_TIMEOUT)
            .build()
            .expect("failed to build HTTP upload client");
        Self {
            http_client: reqwest::Client::new(),
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
    ) -> reqwest::RequestBuilder {
        let url = self.join(path.as_ref());
        let request = self
            .http_client
            .request(method, url)
            .timeout(API_CALL_TIMEOUT)
            .header("X-SDK-Version", env!("CARGO_PKG_VERSION"));

        match &self.auth {
            Auth::None => request,
            Auth::SessionCookie(cookie) => request.header(COOKIE, cookie),
        }
    }

    pub async fn get_json<R>(&self, path: impl AsRef<str>) -> Result<R, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self
            .req(reqwest::Method::GET, path, None::<serde_json::Value>)
            .await?;
        let bytes = response.bytes().await?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub async fn get(&self, path: impl AsRef<str>) -> Result<(), ClientError> {
        self.req(reqwest::Method::GET, path, None::<serde_json::Value>)
            .await
            .map(|_| ())
    }

    pub async fn get_optional_json<R>(
        &self,
        path: impl AsRef<str>,
    ) -> Result<Option<R>, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self
            .req(reqwest::Method::GET, path, None::<serde_json::Value>)
            .await?;
        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let bytes = response.bytes().await?;
        Ok(Some(serde_json::from_slice::<R>(&bytes)?))
    }

    pub async fn post_json<T, R>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::POST, path, body).await?;
        let bytes = response.bytes().await?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub async fn post<T>(&self, path: impl AsRef<str>, body: Option<T>) -> Result<(), ClientError>
    where
        T: serde::Serialize,
    {
        self.req(reqwest::Method::POST, path, body)
            .await
            .map(|_| ())
    }

    pub async fn patch_json<T, R>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::PATCH, path, body).await?;
        let bytes = response.bytes().await?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub async fn delete(&self, path: impl AsRef<str>) -> Result<(), ClientError> {
        self.req(reqwest::Method::DELETE, path, None::<serde_json::Value>)
            .await
            .map(|_| ())
    }

    pub async fn delete_json<R>(&self, path: impl AsRef<str>) -> Result<R, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self
            .req(reqwest::Method::DELETE, path, None::<serde_json::Value>)
            .await?;
        let bytes = response.bytes().await?;
        Ok(serde_json::from_slice::<R>(&bytes)?)
    }

    pub async fn req<T: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<reqwest::Response, ClientError> {
        let request = self.request(method, path);

        let request = if let Some(body) = body {
            request
                .body(serde_json::to_vec(&body)?)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
        } else {
            request
        };

        tracing::debug!("Sending request to Burn API: {:?}", request);
        let response = request.send().await?.map_to_tracel_err().await?;
        tracing::debug!("Received response from Burn API: {:?}", response);

        Ok(response)
    }

    /// Uploads raw bytes to an absolute (presigned) URL via PUT.
    ///
    /// Unlike the other helpers this does NOT join the path with `base_url` and
    /// does NOT attach auth — presigned URLs (e.g. S3) are absolute and
    /// self-authenticating.
    ///
    /// The request is given [a timeout drawn from its own
    /// size](timeout_worth_allowing_an_upload_of) rather than the one API calls
    /// get, which no upload larger than a few megabytes would survive. Its
    /// length is declared up front, since a presigned PUT rejects chunked
    /// transfer encoding.
    pub async fn upload_bytes_to_url(&self, url: &str, bytes: Vec<u8>) -> Result<(), ClientError> {
        let timeout = timeout_worth_allowing_an_upload_of(bytes.len() as u64);

        self.upload_client
            .put(url)
            .timeout(timeout)
            .header(reqwest::header::CONTENT_LENGTH, bytes.len())
            .body(bytes)
            .send()
            .await?
            .map_to_tracel_err()
            .await?;

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

pub trait ResponseExt {
    /// Passes a successful response through and turns any other status into
    /// the [`ClientError`] it stands for, reading the body where it carries
    /// the error code.
    async fn map_to_tracel_err(self) -> Result<reqwest::Response, ClientError>;
}

impl ResponseExt for reqwest::Response {
    async fn map_to_tracel_err(self) -> Result<reqwest::Response, ClientError> {
        if self.status().is_success() {
            return Ok(self);
        }

        match self.status() {
            reqwest::StatusCode::NOT_FOUND => {
                let code = self
                    .text()
                    .await
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
            status => Err(ClientError::ApiError {
                status,
                body: self
                    .text()
                    .await
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn assert_send(_: &impl Send) {}

    #[test]
    fn requests_are_send() {
        let transport = ApiTransport::new(Url::parse("http://localhost").unwrap());

        assert_send(&transport.get_json::<serde_json::Value>("user"));
        assert_send(&transport.post::<()>("logout", None));
        assert_send(&transport.upload_bytes_to_url("http://localhost", Vec::new()));
    }
}
