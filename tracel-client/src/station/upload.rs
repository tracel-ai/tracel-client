use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedUploadUrlResponse {
    pub part: u32,
    pub url: String,
    pub size_bytes: u64,
}
