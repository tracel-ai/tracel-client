use std::collections::HashMap;

use serde::Deserialize;

use super::request::Visibility;

#[derive(Deserialize, Debug)]
pub struct ProjectResponse {
    pub project_name: String,
    pub namespace_name: String,
    pub namespace_type: String,
    pub description: String,
    pub created_by: String,
    #[serde(default)]
    pub visibility: Visibility,
}

pub type ProjectListResponse = Vec<ProjectResponse>;

#[derive(Debug, Deserialize)]
pub struct CodeUploadUrlsResponse {
    pub id: String,
    pub digest: String,
    pub urls: Option<HashMap<String, String>>,
}
