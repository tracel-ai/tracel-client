use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ProjectResponse {
    pub project_name: String,
    pub namespace_name: String,
    pub namespace_type: String,
    pub description: String,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CodeUploadUrlsResponse {
    pub id: String,
    pub digest: String,
    pub uploads: HashMap<String, CodeUploadDescriptor>,
}

/// A multipart upload the server prepared for one artifact.
#[derive(Debug, Deserialize)]
pub struct CodeUploadDescriptor {
    /// The S3 multipart `upload_id`, echoed back to the server at completion.
    pub id: String,
    pub parts: Vec<CodeUploadPart>,
}

/// A single presigned part URL the client PUTs bytes to.
#[derive(Debug, Deserialize)]
pub struct CodeUploadPart {
    pub part: u32,
    pub url: String,
    pub size_bytes: u64,
}
