use reqwest::Url;
use reqwest::header::COOKIE;
use serde::{Deserialize, Serialize};

use crate::credentials::BurnCentralCredentials;
use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        match error.status() {
            Some(status) => ClientError::ApiError {
                status,
                body: ApiErrorBody {
                    code: ApiErrorCode::Unknown,
                    message: error.to_string(),
                },
            },
            None => ClientError::UnknownError(error.to_string()),
        }
    }
}

pub(crate) trait ResponseExt {
    fn map_to_burn_central_err(self) -> Result<reqwest::blocking::Response, ClientError>;
}

impl ResponseExt for reqwest::blocking::Response {
    fn map_to_burn_central_err(self) -> Result<reqwest::blocking::Response, ClientError> {
        if self.status().is_success() {
            Ok(self)
        } else {
            match self.status() {
                reqwest::StatusCode::NOT_FOUND => Err(ClientError::NotFound),
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

/// A client for making HTTP requests to the Burn Central API.
///
/// The client can be used to interact with the Burn Central server, such as creating and starting experiments, saving and loading checkpoints, and uploading logs.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) http_client: reqwest::blocking::Client,
    pub(crate) base_url: Url,
    pub(crate) session_cookie: Option<String>,
    pub(crate) env: Env,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Env {
    Production,
    Staging(u8),
    Development,
}

impl Env {
    pub fn get_url(&self) -> Url {
        match self {
            Env::Production => Url::parse("https://console.tracel.ai/api/").unwrap(),
            Env::Staging(version) => {
                Url::parse(&format!("https://s{}-console.tracel.ai/api/", version)).unwrap()
            }
            Env::Development => Url::parse("http://localhost:9001/").unwrap(),
        }
    }
}

impl Client {
    /// Create a new HttpClient with the given base URL and API key.
    pub fn new(env: Env, credentials: &BurnCentralCredentials) -> Result<Self, ClientError> {
        let mut client = Client {
            http_client: reqwest::blocking::Client::new(),
            base_url: env.get_url(),
            session_cookie: None,
            env,
        };

        let cookie = client.login(credentials)?;
        client.session_cookie = Some(cookie);
        Ok(client)
    }

    #[deprecated]
    /// Please use environment based constructor
    pub fn from_url(url: Url, credentials: &BurnCentralCredentials) -> Result<Self, ClientError> {
        let mut client = Client {
            http_client: reqwest::blocking::Client::new(),
            base_url: url,
            session_cookie: None,
            env: Env::Production,
        };

        let cookie = client.login(credentials)?;
        client.session_cookie = Some(cookie);
        Ok(client)
    }

    #[deprecated]
    /// Please use environment instead of url
    pub fn get_endpoint(&self) -> &Url {
        &self.base_url
    }

    pub fn get_env(&self) -> &Env {
        &self.env
    }

    pub(crate) fn get_json<R>(&self, path: impl AsRef<str>) -> Result<R, ClientError>
    where
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::GET, path, None::<serde_json::Value>)?;
        let bytes = response.bytes()?;
        let json = serde_json::from_slice::<R>(&bytes)?;
        Ok(json)
    }

    pub(crate) fn post_json<T, R>(
        &self,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        let response = self.req(reqwest::Method::POST, path, body)?;
        let bytes = response.bytes()?;
        let json = serde_json::from_slice::<R>(&bytes)?;
        Ok(json)
    }

    pub(crate) fn post<T>(&self, path: impl AsRef<str>, body: Option<T>) -> Result<(), ClientError>
    where
        T: serde::Serialize,
    {
        self.req(reqwest::Method::POST, path, body).map(|_| ())
    }

    pub(crate) fn req<T: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: impl AsRef<str>,
        body: Option<T>,
    ) -> Result<reqwest::blocking::Response, ClientError> {
        let url = self.join(path.as_ref());
        let request_builder = self.http_client.request(method, url);

        let mut request_builder = if let Some(body) = body {
            request_builder
                .body(serde_json::to_vec(&body)?)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
        } else {
            request_builder
        };

        if let Some(cookie) = self.session_cookie.as_ref() {
            request_builder = request_builder.header(COOKIE, cookie);
        }
        request_builder = request_builder.header("X-SDK-Version", env!("CARGO_PKG_VERSION"));

        tracing::debug!("Sending request to Burn Central: {:?}", request_builder);

        let response = request_builder.send()?.map_to_burn_central_err()?;

        tracing::debug!("Received response from Burn Central: {:?}", response);

        Ok(response)
    }

    // Todo update to support multiple versions
    pub(crate) fn join(&self, path: &str) -> Url {
        self.join_versioned(path, 1)
    }

    fn join_versioned(&self, path: &str, version: u8) -> Url {
        self.base_url
            .join(&format!("v{version}/"))
            .unwrap()
            .join(path)
            .expect("Should be able to join url")
    }

    /// Generic method to upload bytes to the given URL.
    pub fn upload_bytes_to_url(&self, url: &str, bytes: Vec<u8>) -> Result<(), ClientError> {
        self.http_client
            .put(url)
            .body(bytes)
            .send()?
            .map_to_burn_central_err()?;

        Ok(())
    }

    /// Generic method to download bytes from the given URL.
    pub fn download_bytes_from_url(&self, url: &str) -> Result<Vec<u8>, ClientError> {
        let data = self
            .http_client
            .get(url)
            .send()?
            .map_to_burn_central_err()?
            .bytes()?
            .to_vec();

        Ok(data)
    }
}
