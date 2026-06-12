use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnotationSetRequest {
    pub name: String,
    pub cleanup_policy: Option<CleanupPolicyRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoteAnnotationSetRequest {
    pub dataset_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAnnotationSetItemsRequest {
    pub items: Vec<AddAnnotationSetItemRequest>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAnnotationSetItemRequest {
    pub source_item_id: Option<String>,
    #[serde_as(as = "serde_with::base64::Base64")]
    pub example_payload: Vec<u8>,
    pub annotation: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateAnnotationSetItemRequest {
    pub validated_annotation: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationSetItemValidationFilterRequest {
    Validated,
    Unvalidated,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnotationSetItemsFilterRequest {
    pub source_item_id: Option<String>,
    pub validation: Option<AnnotationSetItemValidationFilterRequest>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupIntervalUnitRequest {
    Minutes,
    Hours,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CleanupIntervalRequest {
    pub unit: CleanupIntervalUnitRequest,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicyRequest {
    pub target_items: u32,
    pub time_delta: CleanupIntervalRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAnnotationSetCleanupPolicyRequest {
    pub policy: Option<CleanupPolicyRequest>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryAnnotationSetItemsRequest {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    #[serde(default)]
    pub include_data: bool,
    pub filter: Option<AnnotationSetItemsFilterRequest>,
}
