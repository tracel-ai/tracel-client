use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProgressNodeRequest {
    pub id: u64,
    pub parent: Option<u64>,
    pub name: String,
    pub unit: Option<String>,
    pub total: Option<u64>,
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub enum ProgressStatusRequest {
    Success,
    Abandoned,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ProgressEventRequest {
    Started {
        node: ProgressNodeRequest,
    },
    Updated {
        id: u64,
        current: u64,
        total: Option<u64>,
    },
    Message {
        id: u64,
        message: String,
    },
    Finished {
        id: u64,
        status: ProgressStatusRequest,
        message: Option<String>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputUsed {
    Artifact { artifact_id: String },
    Model { model_version_id: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExperimentCompletion {
    Success,
    Fail { reason: String },
}

#[derive(Debug, Serialize)]
pub struct MetricLog {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ExperimentMessage {
    MetricsLog {
        epoch: usize,
        split: String,
        iteration: usize,
        items: Vec<MetricLog>,
    },
    MetricDefinitionLog {
        name: String,
        description: Option<String>,
        unit: Option<String>,
        higher_is_better: bool,
    },
    EpochSummaryLog {
        epoch: usize,
        split: String,
        best_metric_values: Vec<MetricLog>,
    },
    Log(String),
    Arguments(serde_json::Value),
    Config {
        value: serde_json::Value,
        name: String,
    },
    InputUsed(InputUsed),
    Progress(ProgressEventRequest),
    Error(String),
    ExperimentComplete(ExperimentCompletion),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ServerMessage {
    CancelRequested,
}
