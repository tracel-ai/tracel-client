pub mod request;
pub mod response;

pub use request::{
    AddFilesRequest, ArtifactFileSpecRequest, CompleteUploadRequest, CreateArtifactRequest,
    CreateExperimentRequest, ListArtifactsQuery, ListExperimentsQuery, LogUrlsQuery,
    MetricAggregatedQuery, MetricSummaryQuery,
};
pub use response::{
    ArtifactCreationResponse, ArtifactDownloadResponse, ArtifactListResponse, ArtifactResponse,
    DeleteExperimentArtifactResponse, ExperimentInputResponse, ExperimentListResponse,
    ExperimentLogResponse, ExperimentResponse, LoadLogUrlResponse, LoadLogUrlsResponse,
    MetricEntryResponse, MetricGroupResponse, MetricMetadataResponse, MetricResponse,
    MetricSummaryGroupResponse, MetricSummaryResponse, MultipartUploadResponse,
    PresignedArtifactFileUploadUrlsResponse, PresignedArtifactFileUrlResponse,
    PresignedUploadUrlResponse,
};

use crate::{ClientError, WebSocketClient, transport::ApiTransport, websocket::WebSocketError};

pub struct ExperimentClient<'a> {
    transport: &'a ApiTransport,
}

impl<'a> ExperimentClient<'a> {
    pub(crate) fn new(transport: &'a ApiTransport) -> Self {
        Self { transport }
    }

    pub fn create(
        &self,
        request: CreateExperimentRequest,
    ) -> Result<ExperimentResponse, ClientError> {
        self.transport.post_json("experiments", Some(request))
    }

    pub fn get(&self, experiment_num: i32) -> Result<ExperimentResponse, ClientError> {
        self.transport
            .get_json(format!("experiments/{experiment_num}"))
    }

    pub fn list(&self, query: ListExperimentsQuery) -> Result<ExperimentListResponse, ClientError> {
        let mut url = self.transport.join("experiments");
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(page) = query.page {
                pairs.append_pair("page", &page.to_string());
            }
            if let Some(per_page) = query.per_page {
                pairs.append_pair("per_page", &per_page.to_string());
            }
        }

        self.transport.get_json(url)
    }

    pub fn latest(&self) -> Result<Option<ExperimentResponse>, ClientError> {
        self.transport.get_json("experiments/latest")
    }

    pub fn metric_metadata(
        &self,
        experiment_num: i32,
    ) -> Result<MetricMetadataResponse, ClientError> {
        self.transport
            .get_json(format!("experiments/{experiment_num}/metrics/metadata"))
    }

    pub fn metric_summary(
        &self,
        experiment_num: i32,
        query: MetricSummaryQuery,
    ) -> Result<Option<MetricSummaryResponse>, ClientError> {
        let mut url = self
            .transport
            .join(&format!("experiments/{experiment_num}/metrics/summary"));
        url.query_pairs_mut().append_pair("metric", &query.metric);

        self.transport.get_optional_json(url)
    }

    pub fn metrics(
        &self,
        experiment_num: i32,
        query: MetricAggregatedQuery,
    ) -> Result<Option<MetricResponse>, ClientError> {
        let mut url = self
            .transport
            .join(&format!("experiments/{experiment_num}/metrics"));
        url.query_pairs_mut()
            .append_pair("metric", &query.metric)
            .append_pair("max_points", &query.max_points.to_string())
            .append_pair(
                "downsampling_factor",
                &query.downsampling_factor.to_string(),
            );

        self.transport.get_optional_json(url)
    }

    pub fn websocket_url(&self, experiment_num: i32) -> String {
        let mut url = self
            .transport
            .join(&format!("experiments/{experiment_num}/ws"));
        url.set_scheme(if self.transport.base_url().scheme() == "https" {
            "wss"
        } else {
            "ws"
        })
        .expect("Should be able to set ws scheme");

        url.to_string()
    }

    pub fn create_run_websocket(
        &self,
        experiment_num: i32,
    ) -> Result<WebSocketClient, WebSocketError> {
        let mut ws_client = WebSocketClient::new();
        ws_client.connect(&self.websocket_url(experiment_num), self.transport.auth())?;

        Ok(ws_client)
    }

    pub fn cancel(&self, experiment_num: i32) -> Result<(), ClientError> {
        self.transport
            .post(format!("experiments/{experiment_num}/cancel"), None::<()>)
    }

    pub fn create_artifact(
        &self,
        experiment_num: i32,
        request: CreateArtifactRequest,
    ) -> Result<ArtifactCreationResponse, ClientError> {
        self.transport.post_json(
            format!("experiments/{experiment_num}/artifacts"),
            Some(request),
        )
    }

    pub fn add_artifact_files(
        &self,
        experiment_num: i32,
        artifact_id: impl std::fmt::Display,
        request: AddFilesRequest,
    ) -> Result<ArtifactCreationResponse, ClientError> {
        self.transport.post_json(
            format!("experiments/{experiment_num}/artifacts/{artifact_id}/files"),
            Some(request),
        )
    }

    pub fn complete_artifact_upload(
        &self,
        experiment_num: i32,
        artifact_id: impl std::fmt::Display,
        request: CompleteUploadRequest,
    ) -> Result<(), ClientError> {
        self.transport.post(
            format!("experiments/{experiment_num}/artifacts/{artifact_id}/complete"),
            Some(request),
        )
    }

    pub fn presign_artifact_download(
        &self,
        experiment_num: i32,
        artifact_id: impl std::fmt::Display,
    ) -> Result<ArtifactDownloadResponse, ClientError> {
        self.transport.get_json(format!(
            "experiments/{experiment_num}/artifacts/{artifact_id}/download"
        ))
    }

    pub fn list_artifacts(
        &self,
        experiment_num: i32,
        query: ListArtifactsQuery,
    ) -> Result<ArtifactListResponse, ClientError> {
        let mut url = self
            .transport
            .join(&format!("experiments/{experiment_num}/artifacts"));
        if let Some(name) = query.name {
            url.query_pairs_mut().append_pair("name", &name);
        }

        self.transport.get_json(url)
    }

    pub fn delete_artifact(
        &self,
        experiment_num: i32,
        artifact_id: impl std::fmt::Display,
    ) -> Result<DeleteExperimentArtifactResponse, ClientError> {
        self.transport.delete_json(format!(
            "experiments/{experiment_num}/artifacts/{artifact_id}"
        ))
    }

    pub fn realtime_logs(&self, experiment_num: i32) -> Result<ExperimentLogResponse, ClientError> {
        self.transport
            .get_json(format!("experiments/{experiment_num}/logs/realtime"))
    }

    pub fn logs(
        &self,
        experiment_num: i32,
        query: LogUrlsQuery,
    ) -> Result<LoadLogUrlsResponse, ClientError> {
        let mut url = self
            .transport
            .join(&format!("experiments/{experiment_num}/logs"));
        if let Some(start) = query.start {
            url.query_pairs_mut()
                .append_pair("start", &start.to_string());
        }

        self.transport.get_json(url)
    }
}
