use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub(crate) struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RegisteredFunctionRequest {
    pub mod_path: String,
    pub fn_name: String,
    pub proc_type: String,
    pub code: String,
    pub routine: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TracelCodeMetadataRequest {
    pub functions: Vec<RegisteredFunctionRequest>,
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct CodeUploadRequest {
    pub target_package_name: String,
    pub tracel_metadata: TracelCodeMetadataRequest,
    pub crates: Vec<CrateVersionMetadataRequest>,
    pub digest: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CrateVersionMetadataRequest {
    pub checksum: String,
    pub metadata: serde_json::Value,
    pub size: u64,
}
