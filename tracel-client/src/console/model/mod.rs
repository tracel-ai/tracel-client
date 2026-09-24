pub mod request;
pub mod response;

use crate::{
    ClientError,
    console::Client,
    console::model::{
        request::{CreateModelRequest, RequestModelVersionUploadRequest},
        response::{
            ModelDownloadResponse, ModelListResponse, ModelResponse, ModelVersionListResponse,
            ModelVersionResponse, RequestModelVersionUploadResponse,
        },
    },
};

impl Client {
    /// Creates a new model within the specified project.
    ///
    /// The client must be logged in before calling this method.
    pub fn create_model(
        &self,
        namespace: &str,
        project_name: &str,
        req: CreateModelRequest,
    ) -> Result<ModelResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/models"),
            Some(req),
        )
    }

    /// List the models of a project.
    ///
    /// The client must be logged in before calling this method.
    pub fn list_models(
        &self,
        namespace: &str,
        project_name: &str,
    ) -> Result<ModelListResponse, ClientError> {
        self.transport
            .get_json(format!("projects/{namespace}/{project_name}/models"))
    }

    /// List the published versions of a model.
    ///
    /// The client must be logged in before calling this method.
    pub fn list_model_versions(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
    ) -> Result<ModelVersionListResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions"
        ))
    }

    /// Get details about a specific model.
    ///
    /// The client must be logged in before calling this method.
    pub fn get_model(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
    ) -> Result<ModelResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/models/{model_name}"
        ))
    }

    /// Get details about a specific model version.
    ///
    /// The client must be logged in before calling this method.
    pub fn get_model_version(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<ModelVersionResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}"
        ))
    }

    /// Resolves `latest`, a version number (`7` or `v7`) or an alias; only a number can name a
    /// version that is not ready.
    ///
    /// The client must be logged in before calling this method.
    pub fn resolve_model_version_ref(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        reference: &str,
    ) -> Result<ModelVersionResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/refs/{reference}"
        ))
    }

    /// Generate presigned URLs for downloading model version files.
    ///
    /// The client must be logged in before calling this method.
    pub fn presign_model_download(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<ModelDownloadResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}/download"
        ))
    }

    pub fn request_model_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        req: RequestModelVersionUploadRequest,
    ) -> Result<RequestModelVersionUploadResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/models/{model_name}/versions"),
            Some(req),
        )
    }

    pub fn complete_model_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<(), ClientError> {
        self.transport.post(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}/complete"),
            None::<()>
        )
    }
}
