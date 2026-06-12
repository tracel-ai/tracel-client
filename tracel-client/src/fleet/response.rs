use serde::{Deserialize, Serialize};

/// Response containing fleet sync snapshot with model and configuration updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSyncSnapshotResponse {
    /// The model identifier.
    pub model_id: String,
    /// The model version identifier.
    pub model_version_id: String,
    /// Runtime configuration for the device.
    pub runtime_config: serde_json::Value,
}

/// Presigned file descriptor used for fleet model downloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetPresignedModelFileUrlResponse {
    /// Path relative to the model root.
    pub rel_path: String,
    /// Presigned URL from which to download file bytes.
    pub url: String,
    /// Expected file size in bytes.
    pub size_bytes: u64,
    /// Expected checksum (sha256).
    pub checksum: String,
}

/// Response containing presigned URLs for downloading model files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetModelDownloadResponse {
    /// The model version identifier.
    pub model_version_id: String,
    /// List of presigned URLs for each model file.
    pub files: Vec<FleetPresignedModelFileUrlResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetDeviceAuthTokenResponse {
    pub access_token: String,
    pub expires_in_seconds: u64,
}
