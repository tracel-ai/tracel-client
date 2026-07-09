//! Inference-group client: create groups and ingest telemetry.

pub mod request;
pub mod response;

use crate::inference::request::{CreateInferenceGroupRequest, IngestTelemetryRequest};
use crate::inference::response::InferenceGroupResponse;
use crate::{Client, ClientError};

impl Client {
    /// Create a new inference group in the given project.
    ///
    /// The client must be logged in before calling this method.
    pub fn create_inference_group(
        &self,
        owner_name: &str,
        project_name: &str,
        name: String,
        description: Option<String>,
    ) -> Result<InferenceGroupResponse, ClientError> {
        let path = format!("projects/{owner_name}/{project_name}/inference-groups");
        self.transport.post_json(
            path,
            Some(CreateInferenceGroupRequest { name, description }),
        )
    }

    /// Fetch an inference group by name.
    pub fn get_inference_group(
        &self,
        owner_name: &str,
        project_name: &str,
        inference_group_name: &str,
    ) -> Result<InferenceGroupResponse, ClientError> {
        let path =
            format!("projects/{owner_name}/{project_name}/inference-groups/{inference_group_name}");
        self.transport.get_json(path)
    }

    /// Ingest a batch of telemetry (metrics, descriptors and logs) into an inference group.
    pub fn ingest_inference_telemetry(
        &self,
        owner_name: &str,
        project_name: &str,
        inference_group_name: &str,
        telemetry: IngestTelemetryRequest,
    ) -> Result<(), ClientError> {
        let path = format!(
            "projects/{owner_name}/{project_name}/inference-groups/{inference_group_name}/telemetry"
        );
        self.transport.post(path, Some(telemetry))
    }
}
