use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExperimentResponse {
    pub experiment_num: i32,
    pub name: Option<String>,
    pub attributes: HashMap<String, Value>,
}

/// The complete experiment returned by the v1 read endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExperimentDetailsResponse {
    pub id: i32,
    pub experiment_num: i32,
    pub project_id: i32,
    pub name: Option<String>,
    pub status: String,
    pub description: String,
    pub config: Value,
    pub created_by: CreatedByUserResponse,
    pub created_at: String,
    pub configurations: HashMap<String, Value>,
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreatedByUserResponse {
    pub id: i32,
    pub username: String,
    pub namespace: String,
    pub profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListExperimentsResponse {
    pub items: Vec<ExperimentDetailsResponse>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricEntryResponse {
    pub epoch: usize,
    pub iteration: usize,
    pub value: f64,
    pub low: f64,
    pub high: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricGroupResponse {
    pub name: String,
    pub entries: Vec<MetricEntryResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricResponse {
    pub groups: Vec<MetricGroupResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricMetadataResponse {
    pub metric_types: Vec<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricSummaryGroupResponse {
    pub group: String,
    pub optimal_value: f64,
    pub epoch: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricSummaryResponse {
    pub groups: Vec<MetricSummaryGroupResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityMeterResponse {
    pub unit: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityTreeResponse {
    pub roots: Vec<ActivityResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityResponse {
    pub id: u64,
    pub parent: Option<u64>,
    pub name: String,
    pub cancellable: bool,
    pub meter: Option<ActivityMeterResponse>,
    pub current: Option<u64>,
    pub cancel_requested_at: Option<String>,
    pub status: Option<String>,
    pub attributes: Map<String, Value>,
    pub last_message: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
    pub children: Vec<ActivityResponse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevelResponse {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExperimentLogItemResponse {
    /// Sequence number to pass as `after` when tailing logs.
    pub seq: u64,
    pub log_level: LogLevelResponse,
    pub timestamp: String,
    pub message: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExperimentLogQueryResponse {
    pub running: bool,
    pub items: Vec<ExperimentLogItemResponse>,
    pub has_more: bool,
    /// Offset for the next page in time-range mode.
    pub next_cursor: Option<u32>,
    pub total: u64,
}
