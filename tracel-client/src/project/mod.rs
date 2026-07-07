pub mod request;
pub mod response;

use crate::{
    Client, ClientError,
    project::{
        request::{CompleteCodeUploadRequest, CreateProjectRequest, PublishProjectVersionRequest},
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

    /// Request presigned upload URLs for a new code version. The returned
    /// `urls` map is keyed by the binary target-triple string (e.g.
    /// `x86_64-unknown-linux-gnu`) for binaries, or `source.zip` for source.
    /// `urls` is `None` when a version with the same `digest` already exists.
    pub fn publish_project_version_urls(
        &self,
        owner_name: &str,
        project_name: &str,
        request: PublishProjectVersionRequest,
    ) -> Result<CodeUploadUrlsResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{owner_name}/{project_name}/code/upload"),
            Some(request),
        )
    }

    pub fn complete_project_version_upload(
        &self,
        owner_name: &str,
        project_name: &str,
        code_version_id: &str,
        request: CompleteCodeUploadRequest,
    ) -> Result<(), ClientError> {
        self.transport.post(
            format!("projects/{owner_name}/{project_name}/code/{code_version_id}/complete"),
            Some(request),
        )
    }

    /// Upload raw bytes to an absolute presigned upload URL (PUT).
    pub fn upload_bytes_to_url(
        &self,
        url: &str,
        bytes: Vec<u8>,
    ) -> Result<Option<String>, ClientError> {
        self.transport.upload_bytes_to_url(url, bytes)
    }
}
