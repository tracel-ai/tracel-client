pub mod request;
pub mod response;

use reqwest::Url;

use crate::{
    ClientError,
    console::Client,
    console::model::{
        request::{
            CreateModelRequest, ModelVersionListState, PromoteModelVersionRequest,
            RequestModelVersionUploadRequest, SetModelAliasRequest,
        },
        response::{
            ModelAliasListResponse, ModelAliasResponse, ModelDownloadResponse, ModelListResponse,
            ModelResponse, ModelVersionListResponse, ModelVersionResponse,
            RequestModelVersionUploadResponse,
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

    /// List ready model versions or versions in all states.
    ///
    /// The client must be logged in before calling this method.
    pub fn list_model_versions_in_state(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        state: ModelVersionListState,
    ) -> Result<ModelVersionListResponse, ClientError> {
        let mut url = self.transport.join(&format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions"
        ));
        append_version_list_query(state, &mut url);

        self.transport.get_json(url)
    }

    /// Promote an experiment artifact into a new model version.
    ///
    /// The client must be logged in before calling this method.
    pub fn promote_model_version(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        req: PromoteModelVersionRequest,
    ) -> Result<ModelVersionResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/models/{model_name}"),
            Some(req),
        )
    }

    /// List model aliases in name order.
    ///
    /// The client must be logged in before calling this method.
    pub fn list_model_aliases(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
    ) -> Result<ModelAliasListResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/aliases"
        ))
    }

    /// Create or move an alias to a ready model version.
    ///
    /// The client must be logged in before calling this method.
    pub fn set_model_alias(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        alias: &str,
        req: SetModelAliasRequest,
    ) -> Result<ModelAliasResponse, ClientError> {
        self.transport.put_json(
            format!("projects/{namespace}/{project_name}/models/{model_name}/aliases/{alias}"),
            Some(req),
        )
    }

    /// Remove a model alias.
    ///
    /// The client must be logged in before calling this method.
    pub fn remove_model_alias(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        alias: &str,
    ) -> Result<(), ClientError> {
        self.transport.delete(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/aliases/{alias}"
        ))
    }

    /// Replace a model version's metadata with a JSON object or null.
    ///
    /// The client must be logged in before calling this method.
    pub fn update_model_version_metadata(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
        metadata: serde_json::Value,
    ) -> Result<ModelVersionResponse, ClientError> {
        self.transport.put_json(
            format!(
                "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}/metadata"
            ),
            Some(metadata),
        )
    }

    /// Delete a ready or failed model version and its files.
    ///
    /// The client must be logged in before calling this method.
    pub fn delete_model_version(
        &self,
        namespace: &str,
        project_name: &str,
        model_name: &str,
        version: u32,
    ) -> Result<(), ClientError> {
        self.transport.delete(format!(
            "projects/{namespace}/{project_name}/models/{model_name}/versions/{version}"
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

fn append_version_list_query(state: ModelVersionListState, url: &mut Url) {
    let state = match state {
        ModelVersionListState::Ready => "ready",
        ModelVersionListState::All => "all",
    };
    url.query_pairs_mut().append_pair("state", state);
}
