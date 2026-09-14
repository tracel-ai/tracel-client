pub mod request;
pub mod response;

use crate::{
    ClientError,
    console::Client,
    console::project::{
        request::{CreateProjectRequest, PublishProjectVersionRequest, Visibility},
        response::{CodeUploadUrlsResponse, ProjectListResponse, ProjectResponse},
    },
};

impl Client {
    async fn create_project(
        &self,
        project_name: &str,
        project_description: Option<&str>,
        visibility: Visibility,
        path: impl AsRef<str>,
    ) -> Result<ProjectResponse, ClientError> {
        let project_data = CreateProjectRequest {
            name: project_name.to_string(),
            description: project_description.map(|desc| desc.to_string()),
            visibility,
        };

        self.transport.post_json(path, Some(project_data)).await
    }

    pub async fn create_user_project(
        &self,
        project_name: &str,
        project_description: Option<&str>,
        visibility: Visibility,
    ) -> Result<ProjectResponse, ClientError> {
        self.create_project(
            project_name,
            project_description,
            visibility,
            "user/projects",
        )
        .await
    }

    pub async fn get_project(
        &self,
        owner_name: &str,
        project_name: &str,
    ) -> Result<ProjectResponse, ClientError> {
        self.transport
            .get_json(format!("projects/{owner_name}/{project_name}"))
            .await
    }

    /// List the projects owned by a user namespace.
    ///
    /// Only the projects the caller is allowed to see are returned. Fails when the namespace
    /// does not exist.
    pub async fn list_user_projects(
        &self,
        owner_name: &str,
    ) -> Result<ProjectListResponse, ClientError> {
        self.transport
            .get_json(format!("users/{owner_name}/projects"))
            .await
    }

    /// List the projects owned by an organization namespace.
    ///
    /// Only the projects the caller is allowed to see are returned. Fails when the namespace
    /// does not exist.
    pub async fn list_organization_projects(
        &self,
        owner_name: &str,
    ) -> Result<ProjectListResponse, ClientError> {
        self.transport
            .get_json(format!("organizations/{owner_name}/projects"))
            .await
    }

    pub async fn create_organization_project(
        &self,
        owner_name: &str,
        project_name: &str,
        project_description: Option<&str>,
        visibility: Visibility,
    ) -> Result<ProjectResponse, ClientError> {
        self.create_project(
            project_name,
            project_description,
            visibility,
            format!("organizations/{owner_name}/projects"),
        )
        .await
    }

    /// Request presigned upload URLs for a new code version. The returned
    /// `urls` map is keyed by the binary target-triple string (e.g.
    /// `x86_64-unknown-linux-gnu`) for binaries, or `source.zip` for source.
    /// `urls` is `None` when a version with the same `digest` already exists.
    pub async fn publish_project_version_urls(
        &self,
        owner_name: &str,
        project_name: &str,
        request: PublishProjectVersionRequest,
    ) -> Result<CodeUploadUrlsResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{owner_name}/{project_name}/code/upload"),
                Some(request),
            )
            .await
    }

    pub async fn complete_project_version_upload(
        &self,
        owner_name: &str,
        project_name: &str,
        code_version_id: &str,
    ) -> Result<(), ClientError> {
        self.transport
            .post(
                format!("projects/{owner_name}/{project_name}/code/{code_version_id}/complete"),
                None::<()>,
            )
            .await
    }

    /// Upload raw bytes to an absolute presigned upload URL (PUT).
    pub async fn upload_bytes_to_url(&self, url: &str, bytes: Vec<u8>) -> Result<(), ClientError> {
        self.transport.upload_bytes_to_url(url, bytes).await
    }
}
