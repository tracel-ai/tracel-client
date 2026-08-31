use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct DatasetResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatasetListResponse {
    pub items: Vec<DatasetResponse>,
    pub total_count: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatasetVersionResponse {
    pub id: String,
    pub dataset_id: String,
    pub version: i32,
    pub metadata: Option<serde_json::Value>,
    pub source_kind: SourceKindResponse,
    pub created_at: String,
    pub item_count: u64,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKindResponse {
    AnnotationSet,
    DirectUpload,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatasetVersionListResponse {
    pub items: Vec<DatasetVersionResponse>,
    pub total_count: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StartedDatasetVersionUploadResponse {
    pub upload_id: String,
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetVersionUploadItemStatusResponse {
    Inserted,
    Duplicate,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatasetVersionUploadItemOutcomeResponse {
    pub source_item_id: String,
    pub status: DatasetVersionUploadItemStatusResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AddDatasetVersionUploadItemsResponse {
    pub outcomes: Vec<DatasetVersionUploadItemOutcomeResponse>,
}
