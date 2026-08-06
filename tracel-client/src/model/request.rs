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
    /// Opaque, app-defined blob stored with the version, returned verbatim on
    /// reads. Omitted from the request when absent, for servers that predate it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
