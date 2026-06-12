pub mod request;
pub mod response;

use crate::{
    Client, ClientError,
    project::{
        request::{
            CodeUploadRequest, CrateVersionMetadataRequest, CreateProjectRequest,
            TracelCodeMetadataRequest,
        },
        response::{CodeUploadUrlsResponse, ProjectResponse},
    },
};

impl Client {
    fn create_project(
        &self,
        project_name: &str,
        project_description: Option<&str>,
        path: impl AsRef<str>,
    ) -> Result<ProjectResponse, ClientError> {
        let project_data = CreateProjectRequest {
            name: project_name.to_string(),
            description: project_description.map(|desc| desc.to_string()),
        };

        self.transport.post_json(path, Some(project_data))
    }

    pub fn create_user_project(
        &self,
        project_name: &str,
        project_description: Option<&str>,
    ) -> Result<ProjectResponse, ClientError> {
        self.create_project(project_name, project_description, "user/projects")
    }

    pub fn get_project(
        &self,
        owner_name: &str,
        project_name: &str,
    ) -> Result<ProjectResponse, ClientError> {
        self.transport
            .get_json(format!("projects/{owner_name}/{project_name}"))
    }

    pub fn create_organization_project(
        &self,
        owner_name: &str,
        project_name: &str,
        project_description: Option<&str>,
    ) -> Result<ProjectResponse, ClientError> {
        self.create_project(
            project_name,
            project_description,
            format!("organizations/{owner_name}/projects"),
        )
    }

    pub fn publish_project_version_urls(
        &self,
        owner_name: &str,
        project_name: &str,
        target_package_name: &str,
        code_metadata: TracelCodeMetadataRequest,
        crates_metadata: Vec<CrateVersionMetadataRequest>,
        digest: &str,
    ) -> Result<CodeUploadUrlsResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{owner_name}/{project_name}/code/upload"),
            Some(CodeUploadRequest {
                target_package_name: target_package_name.to_string(),
                tracel_metadata: code_metadata,
                crates: crates_metadata,
                digest: digest.to_string(),
            }),
        )
    }

    pub fn complete_project_version_upload(
        &self,
        owner_name: &str,
        project_name: &str,
        code_version_id: &str,
    ) -> Result<(), ClientError> {
        self.transport.post(
            format!("projects/{owner_name}/{project_name}/code/{code_version_id}/complete"),
            None::<()>,
        )
    }
}
