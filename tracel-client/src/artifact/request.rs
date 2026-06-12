use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct ArtifactFileSpecRequest {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CreateArtifactRequest {
    pub name: String,
    pub kind: String,
    pub files: Vec<ArtifactFileSpecRequest>,
}

#[derive(Serialize, Clone, Debug)]
pub(crate) struct AddFilesToArtifactRequest {
    pub files: Vec<ArtifactFileSpecRequest>,
}

#[derive(Serialize, Clone, Debug)]
pub(crate) struct CompleteUploadRequest {
    pub file_names: Option<Vec<String>>,
}
