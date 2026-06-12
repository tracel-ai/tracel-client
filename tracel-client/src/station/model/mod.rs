pub mod request;
pub mod response;

pub use request::{CreateModelRequest, UploadModelFileSpecRequest, UploadModelVersionRequest};
pub use response::{
    ExperimentSourceResponse, FileDescriptorResponse, ModelDownloadResponse, ModelListResponse,
    ModelResponse, ModelVersionListResponse, ModelVersionManifestResponse, ModelVersionResponse,
    PresignedModelFileUploadUrlsResponse, PresignedModelFileUrlResponse,
    PresignedUploadUrlResponse, UploadModelResponse,
};

use crate::{ClientError, transport::ApiTransport};

pub struct ModelClient<'a> {
    transport: &'a ApiTransport,
}

impl<'a> ModelClient<'a> {
    pub(crate) fn new(transport: &'a ApiTransport) -> Self {
        Self { transport }
    }

    pub fn list(&self) -> Result<ModelListResponse, ClientError> {
        self.transport.get_json("models")
    }

    pub fn create(&self, request: CreateModelRequest) -> Result<ModelResponse, ClientError> {
        self.transport.post_json("models", Some(request))
    }

    pub fn get(&self, model_name: &str) -> Result<ModelResponse, ClientError> {
        self.transport.get_json(format!("models/{model_name}"))
    }

    pub fn versions(&self, model_name: &str) -> Result<ModelVersionListResponse, ClientError> {
        self.transport
            .get_json(format!("models/{model_name}/versions"))
    }

    pub fn upload_version(
        &self,
        model_name: &str,
        request: UploadModelVersionRequest,
    ) -> Result<UploadModelResponse, ClientError> {
        self.transport
            .post_json(format!("models/{model_name}/versions"), Some(request))
    }

    pub fn complete_version_upload(
        &self,
        model_name: &str,
        version: u32,
    ) -> Result<(), ClientError> {
        self.transport.post(
            format!("models/{model_name}/versions/{version}/complete"),
            None::<serde_json::Value>,
        )
    }

    pub fn version(
        &self,
        model_name: &str,
        version: u32,
    ) -> Result<ModelVersionResponse, ClientError> {
        self.transport
            .get_json(format!("models/{model_name}/versions/{version}"))
    }

    pub fn download(
        &self,
        model_name: &str,
        version: u32,
    ) -> Result<ModelDownloadResponse, ClientError> {
        self.transport
            .get_json(format!("models/{model_name}/versions/{version}/download"))
    }
}
