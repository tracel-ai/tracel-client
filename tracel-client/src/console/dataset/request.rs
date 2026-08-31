use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CreateDatasetRequest {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct QueryDatasetsRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct QueryDatasetVersionsRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Clone)]
pub struct DatasetVersionUploadItemRequest {
    pub source_item_id: String,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub example_payload: Vec<u8>,
    pub annotation: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AddDatasetVersionUploadItemsRequest {
    pub items: Vec<DatasetVersionUploadItemRequest>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CompleteDatasetVersionUploadRequest {
    pub metadata: Option<serde_json::Value>,
}
