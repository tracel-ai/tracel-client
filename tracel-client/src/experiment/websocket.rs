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
    Failed,
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
#[derive(Debug, Serialize)]
pub struct LogEntry {
    /// RFC3339 / ISO-8601 timestamp (e.g. `2026-04-20T15:10:00Z`).
    pub timestamp: String,
    pub level: LogEntryLevel,
    pub message: String,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    /// Id of the activity this line was emitted under, if any.
    #[serde(default)]
    pub activity: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ExperimentMessage {
    MetricsLog {
        epoch: usize,
        split: String,
        iteration: usize,
        items: Vec<MetricLog>,
        /// Id of the activity this sample was recorded under, if any.
        #[serde(default)]
        activity: Option<u64>,
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
        /// Id of the activity this summary was recorded under, if any.
        #[serde(default)]
        activity: Option<u64>,
    },
    /// Scalar metric summary with no epoch axis.
    SummaryLog {
        items: Vec<MetricLog>,
        /// Id of the activity this summary was recorded under, if any.
        #[serde(default)]
        activity: Option<u64>,
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

    /// Activity attribution must reach the backend on every scoped telemetry message.
    #[test]
    fn telemetry_messages_carry_activity_attribution() {
        let metrics = ExperimentMessage::MetricsLog {
            epoch: 3,
            split: "train".to_string(),
            iteration: 12,
            items: vec![MetricLog {
                name: "loss".to_string(),
                value: 0.25,
            }],
            activity: Some(7),
        };

        assert_eq!(
            serde_json::to_value(&metrics).unwrap(),
            serde_json::json!({
                "type": "metrics_log",
                "data": {
                    "epoch": 3,
                    "split": "train",
                    "iteration": 12,
                    "items": [{ "name": "loss", "value": 0.25 }],
                    "activity": 7,
                },
            })
        );

        let epoch_summary = ExperimentMessage::EpochSummaryLog {
            epoch: 3,
            split: "train".to_string(),
            best_metric_values: vec![],
            activity: None,
        };

        assert_eq!(
            serde_json::to_value(&epoch_summary).unwrap(),
            serde_json::json!({
                "type": "epoch_summary_log",
                "data": {
                    "epoch": 3,
                    "split": "train",
                    "best_metric_values": [],
                    "activity": null,
                },
            })
        );

        let logs = ExperimentMessage::LogEntries(vec![LogEntry {
            timestamp: "2026-04-20T15:10:00Z".to_string(),
            level: LogEntryLevel::Info,
            message: "training".to_string(),
            metadata: serde_json::Map::new(),
            activity: Some(7),
        }]);

        assert_eq!(
            serde_json::to_value(&logs).unwrap(),
            serde_json::json!({
                "type": "log_entries",
                "data": [{
                    "timestamp": "2026-04-20T15:10:00Z",
                    "level": "info",
                    "message": "training",
                    "metadata": {},
                    "activity": 7,
                }],
            })
        );
    }

    /// Scalar summaries have no epoch axis and must not be encoded as epoch summaries.
    #[test]
    fn summary_log_has_no_epoch_axis() {
        let summary = ExperimentMessage::SummaryLog {
            items: vec![MetricLog {
                name: "mean_score".to_string(),
                value: 0.8,
            }],
            activity: Some(4),
        };

        assert_eq!(
            serde_json::to_value(&summary).unwrap(),
            serde_json::json!({
                "type": "summary_log",
                "data": {
                    "items": [{ "name": "mean_score", "value": 0.8 }],
                    "activity": 4,
                },
            })
        );
    }

    /// A failed activity is distinct from an abandoned one and keeps the existing tag casing.
    #[test]
    fn activity_status_failed_serializes_alongside_the_existing_statuses() {
        let statuses = [
            ActivityStatusRequest::Success,
            ActivityStatusRequest::Abandoned,
            ActivityStatusRequest::Failed,
        ];

        assert_eq!(
            serde_json::to_value(statuses).unwrap(),
            serde_json::json!(["Success", "Abandoned", "Failed"])
        );
    }
}
