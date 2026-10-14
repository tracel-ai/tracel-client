use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Debug)]
pub struct CreateModelRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ModelFileSpecRequest {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RequestModelVersionUploadRequest {
    pub files: Vec<ModelFileSpecRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Promote an experiment artifact into a model version.
#[derive(Serialize, Clone, Debug)]
pub struct PromoteModelVersionRequest {
    pub experiment_num: i32,
    /// The artifact UUID.
    pub experiment_file_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Point an alias at a ready model version.
#[derive(Serialize, Clone, Debug)]
pub struct SetModelAliasRequest {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_current_version: Option<u32>,
}

/// Which model versions to list.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelVersionListState {
    Ready,
    All,
}
