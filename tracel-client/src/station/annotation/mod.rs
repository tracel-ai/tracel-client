pub mod request;
pub mod response;

pub use request::{
    AddAnnotationSetItemRequest, AddAnnotationSetItemsRequest,
    AnnotationSetItemValidationFilterRequest, AnnotationSetItemsFilterRequest,
    CleanupIntervalRequest, CleanupIntervalUnitRequest, CleanupPolicyRequest,
    CreateAnnotationSetRequest, PromoteAnnotationSetRequest, QueryAnnotationSetItemsRequest,
    UpdateAnnotationSetCleanupPolicyRequest, ValidateAnnotationSetItemRequest,
};
pub use response::{
    AnnotationSetItemListResponse, AnnotationSetItemResponse, AnnotationSetResponse,
    CleanupIntervalResponse, CleanupIntervalUnitResponse, CleanupPolicyResponse,
    PromotedDatasetVersionResponse,
};

use uuid::Uuid;

use crate::{ClientError, transport::ApiTransport};

pub struct AnnotationClient<'a> {
    transport: &'a ApiTransport,
}

impl<'a> AnnotationClient<'a> {
    pub(crate) fn new(transport: &'a ApiTransport) -> Self {
        Self { transport }
    }

    pub fn create_set(
        &self,
        request: CreateAnnotationSetRequest,
    ) -> Result<AnnotationSetResponse, ClientError> {
        self.transport.post_json("annotation-sets", Some(request))
    }

    pub fn get_set(&self, annotation_set_name: &str) -> Result<AnnotationSetResponse, ClientError> {
        self.transport
            .get_json(format!("annotation-sets/{annotation_set_name}"))
    }

    pub fn update_cleanup_policy(
        &self,
        annotation_set_name: &str,
        request: UpdateAnnotationSetCleanupPolicyRequest,
    ) -> Result<AnnotationSetResponse, ClientError> {
        self.transport.patch_json(
            format!("annotation-sets/{annotation_set_name}"),
            Some(request),
        )
    }

    pub fn promote(
        &self,
        annotation_set_name: &str,
        request: PromoteAnnotationSetRequest,
    ) -> Result<PromotedDatasetVersionResponse, ClientError> {
        self.transport.post_json(
            format!("annotation-sets/{annotation_set_name}/promote"),
            Some(request),
        )
    }

    pub fn add_items(
        &self,
        annotation_set_name: &str,
        request: AddAnnotationSetItemsRequest,
    ) -> Result<(), ClientError> {
        self.transport.post(
            format!("annotation-sets/{annotation_set_name}/items"),
            Some(request),
        )
    }

    pub fn query_items(
        &self,
        annotation_set_name: &str,
        request: QueryAnnotationSetItemsRequest,
    ) -> Result<AnnotationSetItemListResponse, ClientError> {
        self.transport.post_json(
            format!("annotation-sets/{annotation_set_name}/items/query"),
            Some(request),
        )
    }

    pub fn get_item(
        &self,
        annotation_set_name: &str,
        item_id: Uuid,
        include_data: bool,
    ) -> Result<AnnotationSetItemResponse, ClientError> {
        let mut url = self.transport.join(&format!(
            "annotation-sets/{annotation_set_name}/items/{item_id}"
        ));
        url.query_pairs_mut()
            .append_pair("include_data", if include_data { "true" } else { "false" });

        self.transport.get_json(url)
    }

    pub fn validate_item(
        &self,
        annotation_set_name: &str,
        item_id: Uuid,
        request: ValidateAnnotationSetItemRequest,
    ) -> Result<AnnotationSetItemResponse, ClientError> {
        self.transport.patch_json(
            format!("annotation-sets/{annotation_set_name}/items/{item_id}"),
            Some(request),
        )
    }

    pub fn reset_item(
        &self,
        annotation_set_name: &str,
        item_id: Uuid,
    ) -> Result<AnnotationSetItemResponse, ClientError> {
        self.transport.post_json(
            format!("annotation-sets/{annotation_set_name}/items/{item_id}/reset"),
            None::<serde_json::Value>,
        )
    }

    pub fn delete_item(&self, annotation_set_name: &str, item_id: Uuid) -> Result<(), ClientError> {
        self.transport.delete(format!(
            "annotation-sets/{annotation_set_name}/items/{item_id}"
        ))
    }
}
