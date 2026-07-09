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
    Histogram {
        count: u64,
        sum: f64,
        /// Pairs of (upper bucket bound, count in bucket).
        buckets: Vec<(f64, u64)>,
    },
}

/// Kind of a metric, used in metric descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricKind {
    Gauge,
    Counter,
    Histogram,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The serialized payload must match the JSON contract the backend
    /// inference-group telemetry endpoint accepts.
    #[test]
    fn ingest_telemetry_request_matches_backend_contract() {
        let request = IngestTelemetryRequest {
            metrics: vec![
                MetricIngestionEvent {
                    name: "cpu_usage".to_string(),
                    timestamp: "2026-04-20T15:10:00Z".to_string(),
                    metadata: serde_json::json!({ "host": "server1" }),
                    data: MetricData::Gauge { value: 0.75 },
                },
                MetricIngestionEvent {
                    name: "requests_total".to_string(),
                    timestamp: "2026-04-20T15:10:00Z".to_string(),
                    metadata: serde_json::json!({}),
                    data: MetricData::Counter { value: 42 },
                },
                MetricIngestionEvent {
                    name: "latency_ms".to_string(),
                    timestamp: "2026-04-20T15:10:00Z".to_string(),
                    metadata: serde_json::json!({}),
                    data: MetricData::Histogram {
                        count: 3,
                        sum: 12.5,
                        buckets: vec![(1.0, 1), (5.0, 2)],
                    },
                },
            ],
            metric_descriptors: vec![MetricDescriptorEvent {
                name: "cpu_usage".to_string(),
                kind: MetricKind::Gauge,
                unit: Some("percentage".to_string()),
                description: Some("The percentage of CPU usage".to_string()),
            }],
            logs: vec![LogIngestionEvent {
                timestamp: "2026-04-20T15:10:00Z".to_string(),
                level: LogLevel::Error,
                message: "Failed to connect to database".to_string(),
                metadata: serde_json::json!({ "error_code": "ECONNREFUSED" }),
            }],
        };

        let expected = serde_json::json!({
            "metrics": [
                {
                    "name": "cpu_usage",
                    "timestamp": "2026-04-20T15:10:00Z",
                    "metadata": { "host": "server1" },
                    "kind": "gauge",
                    "value": 0.75
                },
                {
                    "name": "requests_total",
                    "timestamp": "2026-04-20T15:10:00Z",
                    "metadata": {},
                    "kind": "counter",
                    "value": 42
                },
                {
                    "name": "latency_ms",
                    "timestamp": "2026-04-20T15:10:00Z",
                    "metadata": {},
                    "kind": "histogram",
                    "count": 3,
                    "sum": 12.5,
                    "buckets": [[1.0, 1], [5.0, 2]]
                }
            ],
            "metric_descriptors": [
                {
                    "name": "cpu_usage",
                    "kind": "gauge",
                    "unit": "percentage",
                    "description": "The percentage of CPU usage"
                }
            ],
            "logs": [
                {
                    "timestamp": "2026-04-20T15:10:00Z",
                    "level": "error",
                    "message": "Failed to connect to database",
                    "metadata": { "error_code": "ECONNREFUSED" }
                }
            ]
        });

        assert_eq!(serde_json::to_value(&request).unwrap(), expected);
    }
}
