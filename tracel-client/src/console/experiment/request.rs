use serde::{Deserialize, Serialize};

/// Query parameters for listing project experiments.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListExperimentsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Sort fields in `field` or `field,asc` / `field,desc` form.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort: Vec<String>,
}

/// Query parameters for one metric series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAggregatedQuery {
    pub metric: String,
    pub max_points: i64,
    pub downsampling_factor: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSummaryQuery {
    pub metric: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevelRequest {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A metadata filter for experiment logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MetadataFilterRequest {
    Equals { key: String, value: String },
    NotEquals { key: String, value: String },
    Exists { key: String },
}

/// Filters and pagination for querying experiment logs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentLogQueryRequest {
    /// Return entries after this sequence, oldest first; ignores time range and offset.
    pub after: Option<u64>,
    /// Start of the time range as a timestamp string.
    pub from: Option<String>,
    /// End of the time range as a timestamp string.
    pub to: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub levels: Option<Vec<LogLevelRequest>>,
    pub search: Option<String>,
    #[serde(default)]
    pub metadata_filters: Vec<MetadataFilterRequest>,
}
