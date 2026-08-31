pub mod request;
pub mod response;

use crate::{
    ClientError,
    console::Client,
    console::dataset::{
        request::{
            AddDatasetVersionUploadItemsRequest, CompleteDatasetVersionUploadRequest,
            CreateDatasetRequest, QueryDatasetVersionsRequest, QueryDatasetsRequest,
        },
        response::{
            AddDatasetVersionUploadItemsResponse, DatasetListResponse, DatasetResponse,
            DatasetVersionItemsPageResponse, DatasetVersionListResponse, DatasetVersionResponse,
            StartedDatasetVersionUploadResponse,
        },
    },
};

impl Client {
    /// Creates a new dataset within the specified project.
    ///
    /// The client must be logged in before calling this method.
    pub fn create_dataset(
        &self,
        namespace: &str,
        project_name: &str,
        req: CreateDatasetRequest,
    ) -> Result<DatasetResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/datasets"),
            Some(req),
        )
    }

    /// Queries the datasets of a project.
    ///
    /// The client must be logged in before calling this method.
    pub fn query_datasets(
        &self,
        namespace: &str,
        project_name: &str,
        req: QueryDatasetsRequest,
    ) -> Result<DatasetListResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/datasets/query"),
            Some(req),
        )
    }

    /// Get details about a specific dataset.
    ///
    /// The client must be logged in before calling this method.
    pub fn get_dataset(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
    ) -> Result<DatasetResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/datasets/{dataset_name}"
        ))
    }

    /// Queries the published versions of a dataset.
    ///
    /// The client must be logged in before calling this method.
    pub fn query_dataset_versions(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        req: QueryDatasetVersionsRequest,
    ) -> Result<DatasetVersionListResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/datasets/{dataset_name}/versions"),
            Some(req),
        )
    }

    /// Starts an upload that becomes a new dataset version.
    ///
    /// The client must be logged in before calling this method.
    pub fn start_dataset_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
    ) -> Result<StartedDatasetVersionUploadResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads"),
            None::<()>,
        )
    }

    /// Appends a batch of items to an upload.
    ///
    /// The client must be logged in before calling this method.
    pub fn add_dataset_version_upload_items(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        upload_id: &str,
        req: AddDatasetVersionUploadItemsRequest,
    ) -> Result<AddDatasetVersionUploadItemsResponse, ClientError> {
        self.transport.post_json(
            format!(
                "projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads/{upload_id}/items"
            ),
            Some(req),
        )
    }

    /// Publishes an upload as a new dataset version.
    ///
    /// The client must be logged in before calling this method.
    pub fn complete_dataset_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        upload_id: &str,
        req: CompleteDatasetVersionUploadRequest,
    ) -> Result<DatasetVersionResponse, ClientError> {
        self.transport.post_json(
            format!(
                "projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads/{upload_id}/complete"
            ),
            Some(req),
        )
    }

    /// Abandons an upload, discarding the items it holds.
    ///
    /// The client must be logged in before calling this method.
    pub fn cancel_dataset_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        upload_id: &str,
    ) -> Result<(), ClientError> {
        self.transport.post(
            format!(
                "projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads/{upload_id}/cancel"
            ),
            None::<()>,
        )
    }

    /// Streams a page of items from a dataset version.
    ///
    /// `index` is where to start, not how many have been seen: resume from the last `entry_idx`
    /// received plus one, since the indices a version holds need not be contiguous.
    ///
    /// The client must be logged in before calling this method.
    pub fn stream_dataset_version_items(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        version: u32,
        index: Option<u64>,
        limit: Option<u32>,
    ) -> Result<DatasetVersionItemsPageResponse, ClientError> {
        let mut query = Vec::new();
        if let Some(index) = index {
            query.push(format!("index={index}"));
        }
        if let Some(limit) = limit {
            query.push(format!("limit={limit}"));
        }
        let query = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };

        self.transport.get_json(format!(
            "projects/{namespace}/{project_name}/datasets/{dataset_name}/versions/{version}/items{query}"
        ))
    }
}
