use serde::{Deserialize, Serialize};

/// Request body to create an inference group in a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInferenceGroupRequest {
    /// Must be URL-safe (no spaces or special characters).
    pub name: String,
    pub description: Option<String>,
}

/// Batch of telemetry posted to an inference group's ingestion endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestTelemetryRequest {
    #[serde(default)]
    pub metrics: Vec<MetricIngestionEvent>,
    #[serde(default)]
    pub metric_descriptors: Vec<MetricDescriptorEvent>,
    #[serde(default)]
    pub logs: Vec<LogIngestionEvent>,
}

/// A single metric sample to ingest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricIngestionEvent {
    pub name: String,
    /// RFC3339 / ISO-8601 timestamp (e.g. `2026-04-20T15:10:00Z`).
    pub timestamp: String,
    pub metadata: serde_json::Value,
    #[serde(flatten)]
    pub data: MetricData,
}

/// Metric payload variants, tagged by `kind` on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum MetricData {
    Gauge {
        value: f64,
    },
    Counter {
        value: u64,
    },
    /// A single raw observation; quantiles are computed server-side at query time.
    Distribution {
        value: f64,
    },
}

/// Kind of a metric, used in metric descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricKind {
    Gauge,
    Counter,
    Distribution,
}

/// Optional descriptor attached to a metric name (unit, description, kind).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDescriptorEvent {
    pub name: String,
    pub kind: MetricKind,
    pub unit: Option<String>,
    pub description: Option<String>,
}

/// Severity of an ingested log line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A single log line to ingest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIngestionEvent {
    /// RFC3339 / ISO-8601 timestamp (e.g. `2026-04-20T15:10:00Z`).
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}
