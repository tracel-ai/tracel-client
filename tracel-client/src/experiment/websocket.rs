use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ActivityMeterRequest {
    pub unit: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ActivityRequest {
    pub id: u64,
    pub parent: Option<u64>,
    pub name: String,
    pub cancellable: bool,
    pub meter: Option<ActivityMeterRequest>,
    #[serde(default)]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub enum ActivityStatusRequest {
    Success,
    Abandoned,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ActivityEventRequest {
    Started {
        activity: ActivityRequest,
    },
    Updated {
        id: u64,
        current: u64,
    },
    Message {
        id: u64,
        message: String,
    },
    Finished {
        id: u64,
        status: ActivityStatusRequest,
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

/// Severity of a [`LogEntry`], serialized as its lowercase name.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogEntryLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A structured log line sent over the experiment websocket.
///
/// The client supplies the timestamp (RFC 3339) and a flat `metadata` object of attributes; the
/// backend stores them alongside the level and message.
#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogEntryLevel,
    pub message: String,
    pub metadata: serde_json::Map<String, serde_json::Value>,
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
    LogEntries(Vec<LogEntry>),
    Arguments(serde_json::Value),
    Config {
        value: serde_json::Value,
        name: String,
    },
    Attribute {
        key: String,
        value: serde_json::Value,
    },
    InputUsed(InputUsed),
    Activity(ActivityEventRequest),
    Error(String),
    ExperimentComplete(ExperimentCompletion),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ServerMessage {
    CancelRequested,
    ActivityCancelRequested { id: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entries_message_matches_backend_contract() {
        let mut metadata = serde_json::Map::new();
        metadata.insert("split".to_string(), serde_json::json!("train"));
        metadata.insert("epoch".to_string(), serde_json::json!(3));

        let message = ExperimentMessage::LogEntries(vec![
            LogEntry {
                timestamp: "2026-04-20T15:10:00Z".to_string(),
                level: LogEntryLevel::Warn,
                message: "slow step".to_string(),
                metadata,
            },
            LogEntry {
                timestamp: "2026-04-20T15:10:01Z".to_string(),
                level: LogEntryLevel::Info,
                message: "done".to_string(),
                metadata: serde_json::Map::new(),
            },
        ]);

        let expected = serde_json::json!({
            "type": "log_entries",
            "data": [
                {
                    "timestamp": "2026-04-20T15:10:00Z",
                    "level": "warn",
                    "message": "slow step",
                    "metadata": { "split": "train", "epoch": 3 }
                },
                {
                    "timestamp": "2026-04-20T15:10:01Z",
                    "level": "info",
                    "message": "done",
                    "metadata": {}
                }
            ]
        });

        assert_eq!(serde_json::to_value(&message).unwrap(), expected);
    }
}
