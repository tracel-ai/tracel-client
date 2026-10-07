use serde::Serialize;

use super::request::ArtifactFileSpecRequest;

#[derive(Serialize, Clone, Debug)]
pub struct AddFilesToArtifactRequest {
    pub files: Vec<ArtifactFileSpecRequest>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CompleteUploadRequest {
    pub file_names: Option<Vec<String>>,
}
