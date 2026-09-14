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
    pub async fn create_dataset(
        &self,
        namespace: &str,
        project_name: &str,
        req: CreateDatasetRequest,
    ) -> Result<DatasetResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{namespace}/{project_name}/datasets"),
                Some(req),
            )
            .await
    }

    /// Queries the datasets of a project.
    ///
    /// The client must be logged in before calling this method.
    pub async fn query_datasets(
        &self,
        namespace: &str,
        project_name: &str,
        req: QueryDatasetsRequest,
    ) -> Result<DatasetListResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{namespace}/{project_name}/datasets/query"),
                Some(req),
            )
            .await
    }

    /// Get details about a specific dataset.
    ///
    /// The client must be logged in before calling this method.
    pub async fn get_dataset(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
    ) -> Result<DatasetResponse, ClientError> {
        self.transport
            .get_json(format!(
                "projects/{namespace}/{project_name}/datasets/{dataset_name}"
            ))
            .await
    }

    /// Queries the published versions of a dataset.
    ///
    /// The client must be logged in before calling this method.
    pub async fn query_dataset_versions(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        req: QueryDatasetVersionsRequest,
    ) -> Result<DatasetVersionListResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{namespace}/{project_name}/datasets/{dataset_name}/versions"),
                Some(req),
            )
            .await
    }

    /// Starts an upload that becomes a new dataset version.
    ///
    /// The client must be logged in before calling this method.
    pub async fn start_dataset_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
    ) -> Result<StartedDatasetVersionUploadResponse, ClientError> {
        self.transport
            .post_json(
                format!("projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads"),
                None::<()>,
            )
            .await
    }

    /// Appends a batch of items to an upload.
    ///
    /// The client must be logged in before calling this method.
    pub async fn add_dataset_version_upload_items(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        upload_id: &str,
        req: AddDatasetVersionUploadItemsRequest,
    ) -> Result<AddDatasetVersionUploadItemsResponse, ClientError> {
        let path = format!(
            "projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads/{upload_id}/items"
        );

        self.transport.post_json(path, Some(req)).await
    }

    /// Publishes an upload as a new dataset version.
    ///
    /// The client must be logged in before calling this method.
    pub async fn complete_dataset_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        upload_id: &str,
        req: CompleteDatasetVersionUploadRequest,
    ) -> Result<DatasetVersionResponse, ClientError> {
        let path = format!(
            "projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads/{upload_id}/complete"
        );

        self.transport.post_json(path, Some(req)).await
    }

    /// Abandons an upload, discarding the items it holds.
    ///
    /// The client must be logged in before calling this method.
    pub async fn cancel_dataset_version_upload(
        &self,
        namespace: &str,
        project_name: &str,
        dataset_name: &str,
        upload_id: &str,
    ) -> Result<(), ClientError> {
        let path = format!(
            "projects/{namespace}/{project_name}/datasets/{dataset_name}/uploads/{upload_id}/cancel"
        );

        self.transport.post(path, None::<()>).await
    }

    /// Streams a page of items from a dataset version.
    ///
    /// `index` is where to start, not how many have been seen: resume from the last `entry_idx`
    /// received plus one, since the indices a version holds need not be contiguous.
    ///
    /// The client must be logged in before calling this method.
    pub async fn stream_dataset_version_items(
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

        let path = format!(
            "projects/{namespace}/{project_name}/datasets/{dataset_name}/versions/{version}/items{query}"
        );

        self.transport.get_json(path).await
    }
}
