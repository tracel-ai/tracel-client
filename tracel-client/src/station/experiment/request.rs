use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExperimentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListExperimentsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSummaryQuery {
    pub metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAggregatedQuery {
    pub metric: String,
    pub max_points: i64,
    pub downsampling_factor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactFileSpecRequest {
    pub rel_path: String,
    pub size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArtifactRequest {
    pub name: String,
    pub kind: String,
    pub files: Vec<ArtifactFileSpecRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFilesRequest {
    pub files: Vec<ArtifactFileSpecRequest>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompleteUploadRequest {
    pub file_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListArtifactsQuery {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogUrlsQuery {
    pub start: Option<u64>,
}
