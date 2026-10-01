use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::console::credentials::TracelCredentials;
use crate::console::session::authenticate;
use crate::console::user::response::UserResponseSchema;
use crate::error::ClientError;
use crate::transport::ApiTransport;

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
    /// Every kind is sent as a bearer token, and the authenticated user is read
    /// back, so the returned client is known to work. Fails with
    /// [`ClientError::Unauthenticated`] if the server rejects the credential,
    /// and with [`ClientError::AppSessionEnded`] if an app session can no
    /// longer be renewed.
    pub fn connect(env: Env, credentials: &TracelCredentials) -> Result<Self, ClientError> {
        Self::connect_to(env.get_url(), env, credentials)
    }

    fn connect_to(
        url: Url,
        env: Env,
        credentials: &TracelCredentials,
    ) -> Result<Self, ClientError> {
        let mut transport = ApiTransport::new(url);
        transport.set_auth(authenticate(credentials)?);

        let url = transport.join("user");
        let user = transport.get_json::<UserResponseSchema>(url)?;

        Ok(Client {
            transport,
            env,
            user,
        })
    }

    /// Connects to a custom base URL and verifies the credentials.
    ///
    /// For servers [`Env`] cannot name, such as a local devstack. Behaves like
    /// [`connect`](Client::connect) otherwise.
    pub fn from_url(url: Url, credentials: &TracelCredentials) -> Result<Self, ClientError> {
        Self::connect_to(url, Env::Production, credentials)
    }

    #[deprecated]
    /// Please use environment instead of url
    pub fn get_endpoint(&self) -> &Url {
        self.transport.base_url()
    }

    /// The base URL every request is resolved against.
    pub fn base_url(&self) -> &Url {
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
