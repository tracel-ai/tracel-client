use serde::Deserialize;

pub use crate::station::upload::PresignedUploadUrlResponse;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub version_count: u64,
    pub latest_version: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentSourceResponse {
    pub id: i32,
    pub experiment_num: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelVersionResponse {
    pub id: String,
    pub experiment: Option<ExperimentSourceResponse>,
    pub version: u32,
    pub state: ModelVersionStateResponse,
    /// Set on a failed version: `upload_expired`, `upload_incomplete`, `promotion_failed` or
    /// `promotion_expired`.
    pub failure_reason: Option<String>,
    pub source_kind: ModelVersionSourceKindResponse,
    pub size: u64,
    /// The sha256 over the manifest's sorted `rel_path:checksum` lines, never over metadata.
    pub digest: String,
    pub aliases: Vec<String>,
    pub created_at: String,
    pub manifest: ModelVersionManifestResponse,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub deleted_at: Option<String>,
}

/// Only a ready version is listed by default, downloadable, `latest` or an alias target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelVersionStateResponse {
    Pending,
    Ready,
    Failed,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelVersionSourceKindResponse {
    Upload,
    Promotion,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelVersionManifestResponse {
    pub files: Vec<FileDescriptorResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileDescriptorResponse {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelListResponse {
    pub items: Vec<ModelResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelVersionListResponse {
    pub items: Vec<ModelVersionResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedModelFileUrlResponse {
    pub rel_path: String,
    pub url: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDownloadResponse {
    pub files: Vec<PresignedModelFileUrlResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadModelResponse {
    pub version: u32,
    pub files: Vec<PresignedModelFileUploadUrlsResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedModelFileUploadUrlsResponse {
    pub rel_path: String,
    pub parts: Vec<PresignedUploadUrlResponse>,
}
