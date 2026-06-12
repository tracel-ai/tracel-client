use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupIntervalUnitResponse {
    Minutes,
    Hours,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CleanupIntervalResponse {
    pub unit: CleanupIntervalUnitResponse,
    pub amount: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CleanupPolicyResponse {
    pub target_items: u32,
    pub time_delta: CleanupIntervalResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnnotationSetResponse {
    pub id: String,
    pub name: String,
    pub cleanup_policy: Option<CleanupPolicyResponse>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct AnnotationSetItemResponse {
    pub id: String,
    pub source_item_id: Option<String>,
    #[serde_as(as = "Option<serde_with::base64::Base64>")]
    pub example_payload: Option<Vec<u8>>,
    pub annotation: Option<serde_json::Value>,
    pub modified_annotation: Option<serde_json::Value>,
    pub effective_annotation: Option<serde_json::Value>,
    pub validated: bool,
    pub example_size_bytes: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnnotationSetItemListResponse {
    pub items: Vec<AnnotationSetItemResponse>,
    pub total_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromotedDatasetVersionResponse {
    pub id: String,
    pub version: i32,
}
