use serde::Serialize;

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
}
