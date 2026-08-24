mod artifact;
pub mod auth;
mod client;
mod credentials;
mod error;
mod experiment;
pub mod inference;
mod job;
mod model;
mod project;
mod session;
mod transport;
mod user;

#[cfg(feature = "station")]
pub mod fleet;
#[cfg(feature = "station")]
pub mod station;
pub mod websocket;

#[cfg(feature = "station")]
pub use station::StationClient;

#[cfg(feature = "tracel")]
mod tracel {
    use super::*;
    pub use credentials::TracelCredentials;
    #[cfg(feature = "station")]
    pub use fleet::FleetClient;

    pub mod response {
        pub use crate::artifact::response::*;
        pub use crate::experiment::response::*;
        #[cfg(feature = "station")]
        pub use crate::fleet::response::*;
        pub use crate::model::response::*;
        pub use crate::project::response::*;
        pub use crate::user::response::*;
    }

    pub mod request {
        pub use crate::artifact::request::*;
        #[cfg(feature = "station")]
        pub use crate::fleet::request::*;
        pub use crate::model::request::*;
        pub use crate::project::request::*;
    }
}

#[cfg(feature = "tracel")]
pub use tracel::*;

#[cfg(feature = "tracel")]
pub use client::Client;

pub use client::Env;
pub use credentials::SessionToken;
pub use error::{ApiErrorCode, ClientError};

pub use websocket::WebSocketClient;
