use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct CreatedByUserResponse {
    pub id: i32,
    pub username: String,
    pub namespace: String,
}

#[derive(Deserialize, Clone, Debug)]
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
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub manifest: ModelVersionManifestResponse,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub deleted_at: Option<String>,
}

/// Only a ready version is listed by default, downloadable, `latest` or an alias target.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelVersionStateResponse {
    Pending,
    Ready,
    Failed,
    Deleted,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelVersionSourceKindResponse {
    Upload,
    Promotion,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelVersionManifestResponse {
    pub files: Vec<ModelFileResponse>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelFileResponse {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ExperimentSourceResponse {
    pub id: i32,
    pub experiment_num: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelResponse {
    pub id: String,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub version_count: u64,
    /// The highest published version number, or `None` when the model has no
    /// versions yet.
    #[serde(default)]
    pub latest_version: Option<u32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelListResponse {
    pub items: Vec<ModelResponse>,
    pub total: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelVersionListResponse {
    pub items: Vec<ModelVersionResponse>,
    pub total: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelDownloadResponse {
    pub files: Vec<PresignedModelFileUrlResponse>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PresignedModelFileUrlResponse {
    pub rel_path: String,
    pub url: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub checksum: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PresignedModelFileUploadUrlsResponse {
    pub rel_path: String,
    pub urls: crate::console::artifact::response::MultipartUploadResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RequestModelVersionUploadResponse {
    pub version: u32,
    pub files: Vec<PresignedModelFileUploadUrlsResponse>,
}
