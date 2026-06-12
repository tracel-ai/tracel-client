use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DatasetQueryFilterRequest {
    #[serde(default)]
    pub metadata: Vec<DatasetMetadataFilterRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetMetadataFilterRequest {
    Equals(DatasetMetadataJsonComparisonRequest),
    Contains(DatasetMetadataJsonComparisonRequest),
    Exists(DatasetMetadataPathRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetMetadataPathRequest {
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetMetadataJsonComparisonRequest {
    pub key: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryDatasetsRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    #[serde(default)]
    pub filter: Option<DatasetQueryFilterRequest>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryDatasetVersionsRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamDatasetVersionItemsRequest {
    pub cursor: Option<u64>,
    pub limit: Option<u32>,
}
