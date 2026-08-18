use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::credentials::TracelCredentials;
use crate::error::{ApiErrorBody, ApiErrorCode, ClientError};
use crate::session::authenticate;
use crate::transport::ApiTransport;
use crate::user::response::UserResponseSchema;

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

/// A client for making HTTP requests to the Tracel API.
///
/// The client can be used to interact with the Tracel server, such as creating and starting experiments, saving and loading checkpoints, and uploading logs.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) transport: ApiTransport,
    pub(crate) env: Env,
    user: UserResponseSchema,
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
    /// Connects to the Tracel server and verifies the credentials.
    ///
    /// An API key is exchanged for a session; a session token is used as-is.
    /// Both paths then read back the authenticated user, so the returned client
    /// is known to work. Fails with [`ClientError::Unauthorized`] if the
    /// credentials are rejected.
    pub fn connect(env: Env, credentials: &TracelCredentials) -> Result<Self, ClientError> {
        Self::connect_to(env.get_url(), env, credentials)
    }

    fn connect_to(
        url: Url,
        env: Env,
        credentials: &TracelCredentials,
    ) -> Result<Self, ClientError> {
        let mut transport = ApiTransport::new(url);
        transport.set_auth(authenticate(&transport, credentials)?);

        // Proves the session is live. The endpoint answers 200 with a `null`
        // body rather than 401 when it is not.
        let url = transport.join("user");
        let user = transport
            .get_json::<Option<UserResponseSchema>>(url)?
            .ok_or(ClientError::Unauthorized)?;

        Ok(Client {
            transport,
            env,
            user,
        })
    }

    #[deprecated]
    /// Please use environment based constructor
    pub fn from_url(url: Url, credentials: &TracelCredentials) -> Result<Self, ClientError> {
        Self::connect_to(url, Env::Production, credentials)
    }

    #[deprecated]
    /// Please use environment instead of url
    pub fn get_endpoint(&self) -> &Url {
        self.transport.base_url()
    }

    pub fn get_env(&self) -> &Env {
        &self.env
    }

    /// The user this client is authenticated as.
    ///
    /// Read once on connect, so this costs no request. Use
    /// [`get_current_user`](Client::get_current_user) to refresh it.
    pub fn user(&self) -> &UserResponseSchema {
        &self.user
    }
}
