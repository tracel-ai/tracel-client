use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::request::Visibility;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectResponse {
    pub project_name: String,
    pub namespace_name: String,
    pub namespace_type: String,
    pub description: String,
    pub created_by: String,
    #[serde(default)]
    pub created_at: String,
    pub visibility: Visibility,
}

pub type ProjectListResponse = Vec<ProjectResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeUploadUrlsResponse {
    pub id: String,
    pub digest: String,
    pub urls: Option<HashMap<String, String>>,
}
