mod artifact;
mod client;
mod credentials;
mod error;
mod experiment;
mod job;
mod model;
mod project;
mod transport;
mod user;

#[cfg(feature = "station")]
pub mod fleet;
#[cfg(feature = "station")]
pub mod station;
pub mod websocket;

#[cfg(feature = "station")]
pub use station::StationClient;

#[cfg(feature = "station")]
mod tracel {
    use super::*;
    pub use credentials::TracelCredentials;
    pub use fleet::FleetClient;

    pub mod response {
        pub use crate::artifact::response::*;
        pub use crate::experiment::response::*;
        pub use crate::fleet::response::*;
        pub use crate::model::response::*;
        pub use crate::project::response::*;
        pub use crate::user::response::*;
    }

    pub mod request {
        pub use crate::artifact::request::*;
        pub use crate::fleet::request::*;
        pub use crate::project::request::*;
    }
}

#[cfg(feature = "tracel")]
pub use tracel::*;

#[cfg(feature = "tracel")]
pub use client::Client;

pub use client::Env;
pub use error::ClientError;

pub use websocket::WebSocketClient;
