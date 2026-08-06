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

/// Project listings are returned as a bare array, not as a paginated envelope.
pub type ProjectListResponse = Vec<ProjectResponse>;

#[derive(Debug, Deserialize)]
pub struct CodeUploadUrlsResponse {
    pub id: String,
    pub digest: String,
    pub urls: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_list_matches_backend_contract() {
        let projects: ProjectListResponse = serde_json::from_str(
            r#"[{
                "project_name": "vision",
                "namespace_name": "ada",
                "namespace_type": "user",
                "description": "",
                "created_by": "ada",
                "created_at": "2026-03-05T18:45:43.397",
                "visibility": "public"
            }]"#,
        )
        .unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].project_name, "vision");
        assert_eq!(projects[0].namespace_name, "ada");
        assert_eq!(projects[0].namespace_type, "user");
    }
}
