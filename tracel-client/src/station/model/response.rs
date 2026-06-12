use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub version_count: u64,
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
    pub size: u64,
    pub checksum: String,
    pub created_at: String,
    pub manifest: ModelVersionManifestResponse,
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

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedUploadUrlResponse {
    pub part: u32,
    pub url: String,
    pub size_bytes: u64,
}
