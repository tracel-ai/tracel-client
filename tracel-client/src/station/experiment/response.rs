use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentResponse {
    pub id: i32,
    pub experiment_num: i32,
    pub name: Option<String>,
    pub status: String,
    pub description: String,
    pub created_at: String,
    pub arguments: Value,
    pub inputs: Vec<ExperimentInputResponse>,
    pub configurations: HashMap<String, Value>,
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExperimentInputResponse {
    Artifact { artifact_id: String },
    Model { model_version_id: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentListResponse {
    pub items: Vec<ExperimentResponse>,
    pub total: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricMetadataResponse {
    pub metric_types: Vec<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricSummaryGroupResponse {
    pub group: String,
    pub optimal_value: f64,
    pub epoch: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricSummaryResponse {
    pub groups: Vec<MetricSummaryGroupResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricEntryResponse {
    pub epoch: usize,
    pub iteration: usize,
    pub value: f64,
    pub low: f64,
    pub high: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricGroupResponse {
    pub name: String,
    pub entries: Vec<MetricEntryResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricResponse {
    pub groups: Vec<MetricGroupResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedUploadUrlResponse {
    pub part: u32,
    pub url: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultipartUploadResponse {
    pub id: String,
    pub parts: Vec<PresignedUploadUrlResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedArtifactFileUploadUrlsResponse {
    pub rel_path: String,
    pub urls: MultipartUploadResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactCreationResponse {
    pub id: String,
    pub files: Vec<PresignedArtifactFileUploadUrlsResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresignedArtifactFileUrlResponse {
    pub rel_path: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactDownloadResponse {
    pub files: Vec<PresignedArtifactFileUrlResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactResponse {
    pub id: String,
    pub created_at: String,
    pub name: String,
    pub kind: String,
    pub experiment_num: i32,
    pub manifest: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactListResponse {
    pub items: Vec<ArtifactResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteExperimentArtifactResponse {
    pub id: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentLogResponse {
    pub running: bool,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadLogUrlResponse {
    pub url: String,
    pub size_header: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadLogUrlsResponse {
    pub urls: Vec<LoadLogUrlResponse>,
    pub size: u64,
}
