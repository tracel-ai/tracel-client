use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct CreatedByUserResponse {
    pub id: i32,
    pub username: String,
    pub namespace: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelVersionResponse {
    pub id: String,
    pub experiment: Option<ExperimentSourceResponse>,
    pub version: u32,
    pub size: u64,
    pub checksum: String,
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub manifest: serde_json::Value,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ExperimentSourceResponse {
    pub id: i32,
    pub experiment_num: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelResponse {
    pub id: String,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub version_count: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ModelDownloadResponse {
    pub files: Vec<PresignedModelFileUrlResponse>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PresignedModelFileUrlResponse {
    pub rel_path: String,
    pub url: String,
}
