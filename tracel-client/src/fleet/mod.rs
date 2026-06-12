//! Fleet management client for Tracel.

pub mod request;
pub mod response;

use crate::ClientError;
use crate::Env;
use crate::request::ExchangeFleetDeviceTokenRequest;
use crate::response::FleetDeviceAuthTokenResponse;
use crate::transport::{ApiTransport, Auth};
use request::{
    DownloadModelRequest, IngestTelemetryRequest, SyncDeviceRequest, TelemetryIngestionEvents,
};
use reqwest::Url;
use response::{FleetModelDownloadResponse, FleetSyncSnapshotResponse};

/// A client for interacting with the Tracel Fleet API.
#[derive(Debug, Clone)]
pub struct FleetClient {
    transport: ApiTransport,
}

impl FleetClient {
    /// Create a new FleetClient for the given environment.
    pub fn new(env: Env) -> Self {
        FleetClient {
            transport: ApiTransport::new(env.get_url()),
        }
    }

    /// Create a FleetClient with a custom base URL.
    pub fn from_url(url: Url) -> Self {
        FleetClient {
            transport: ApiTransport::new(url),
        }
    }

    /// Register the device and exchange credentials for a JWT.
    pub fn register(
        &self,
        registration_token: impl Into<String>,
        identity_key: impl Into<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<FleetDeviceAuthTokenResponse, ClientError> {
        let request = ExchangeFleetDeviceTokenRequest {
            registration_token: registration_token.into(),
            identity_key: identity_key.into(),
            metadata,
        };

        self.post_json("fleets/device/register", Some(request))
    }

    /// Sync device state with the fleet.
    pub fn sync(
        &self,
        token: impl AsRef<str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<FleetSyncSnapshotResponse, ClientError> {
        let request = SyncDeviceRequest { metadata };

        self.with_bearer_auth(token)
            .post_json("fleets/device/sync", Some(request))
    }

    /// Get the model download information for the device's assigned fleet.
    pub fn model_download(
        &self,
        auth_token: impl AsRef<str>,
    ) -> Result<FleetModelDownloadResponse, ClientError> {
        let request = DownloadModelRequest {};

        self.with_bearer_auth(auth_token)
            .post_json("fleets/device/model/download", Some(request))
    }

    /// Ingest telemetry events for a fleet device.
    pub fn ingest_telemetry(
        &self,
        auth_token: impl AsRef<str>,
        events: TelemetryIngestionEvents,
    ) -> Result<(), ClientError> {
        let request = IngestTelemetryRequest { events };

        self.with_bearer_auth(auth_token)
            .post("fleets/device/telemetry", Some(request))
    }

    fn post_json<T, R>(&self, path: impl AsRef<str>, body: Option<T>) -> Result<R, ClientError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        self.transport.post_json(path, body)
    }

    fn with_bearer_auth(&self, auth_token: impl AsRef<str>) -> ApiTransport {
        self.transport
            .clone()
            .with_auth(Auth::Bearer(auth_token.as_ref().to_string()))
    }
}
