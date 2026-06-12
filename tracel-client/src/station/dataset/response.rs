use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetListResponse {
    pub items: Vec<DatasetResponse>,
    pub total_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKindResponse {
    AnnotationSet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetVersionResponse {
    pub id: String,
    pub dataset_id: String,
    pub version: i32,
    pub metadata: Option<serde_json::Value>,
    pub source_kind: SourceKindResponse,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetVersionListResponse {
    pub items: Vec<DatasetVersionResponse>,
    pub total_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetDownloadFileResponse {
    pub rel_path: String,
    pub url: String,
    pub size_bytes: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetDownloadResponse {
    pub files: Vec<DatasetDownloadFileResponse>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct DatasetVersionItemResponse {
    pub entry_idx: u64,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetVersionItemsPageResponse {
    pub items: Vec<DatasetVersionItemResponse>,
    pub next_cursor: Option<u64>,
}
