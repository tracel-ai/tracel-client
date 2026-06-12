pub mod response;

use crate::{
    Client, ClientError,
    model::response::{ModelDownloadResponse, ModelResponse, ModelVersionResponse},
};

impl Client {
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
}
