use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadModelVersionRequest {
    pub files: Vec<UploadModelFileSpecRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadModelFileSpecRequest {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}
