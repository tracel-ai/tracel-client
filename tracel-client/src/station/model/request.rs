use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadModelVersionRequest {
    pub files: Vec<UploadModelFileSpecRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadModelFileSpecRequest {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}
