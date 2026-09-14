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
    pub async fn create_model(
        &self,
        namespace: &str,
        project_name: &str,
        req: CreateModelRequest,
    ) -> Result<ModelResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{namespace}/{project_name}/models"),
                Some(req),
            )
            .await
    }

    /// List the models of a project.
    ///
    /// The client must be logged in before calling this method.
    pub async fn list_models(
        &self,
        namespace: &str,
        project_name: &str,
    ) -> Result<ModelListResponse, ClientError> {
        self.transport
            .get_json(format!("projects/{namespace}/{project_name}/models"))
            .await
    }

    /// List the published versions of a model.
    ///
    /// The client must be logged in before calling this method.
    pub async fn list_model_versions(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
    ) -> Result<ModelVersionListResponse, ClientError> {
        self.transport
            .get_json(format!(
                "projects/{namespace}/{project_name}/models/{model_name}/versions"
            ))
            .await
    }

    /// Get details about a specific model.
    ///
    /// The client must be logged in before calling this method.
    pub async fn get_model(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
    ) -> Result<ModelResponse, ClientError> {
        self.transport
            .get_json(format!(
                "projects/{namespace}/{project_name}/models/{model_name}"
            ))
            .await
    }

    /// Get details about a specific model version.
    ///
    /// The client must be logged in before calling this method.
    pub async fn get_model_version(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<ModelVersionResponse, ClientError> {
        self.transport
            .get_json(format!(
                "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}"
            ))
            .await
    }

    /// Generate presigned URLs for downloading model version files.
    ///
    /// The client must be logged in before calling this method.
    pub async fn presign_model_download(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<ModelDownloadResponse, ClientError> {
        let path = format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}/download"
        );

        self.transport.get_json(path).await
    }

    pub async fn request_model_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        req: RequestModelVersionUploadRequest,
    ) -> Result<RequestModelVersionUploadResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{namespace}/{project_name}/models/{model_name}/versions"),
                Some(req),
            )
            .await
    }

    pub async fn complete_model_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<(), ClientError> {
        let path = format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}/complete"
        );

        self.transport.post(path, None::<()>).await
    }
}
