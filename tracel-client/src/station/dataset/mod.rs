pub mod request;
pub mod response;

pub use request::{
    CreateDatasetRequest, DatasetMetadataFilterRequest, DatasetMetadataJsonComparisonRequest,
    DatasetMetadataPathRequest, DatasetQueryFilterRequest, QueryDatasetVersionsRequest,
    QueryDatasetsRequest, StreamDatasetVersionItemsRequest,
};
pub use response::{
    DatasetDownloadFileResponse, DatasetDownloadResponse, DatasetListResponse, DatasetResponse,
    DatasetVersionItemResponse, DatasetVersionItemsPageResponse, DatasetVersionListResponse,
    DatasetVersionResponse, SourceKindResponse,
};

use crate::{ClientError, transport::ApiTransport};

pub struct DatasetClient<'a> {
    transport: &'a ApiTransport,
}

impl<'a> DatasetClient<'a> {
    pub(crate) fn new(transport: &'a ApiTransport) -> Self {
        Self { transport }
    }

    pub async fn create(
        &self,
        request: CreateDatasetRequest,
    ) -> Result<DatasetResponse, ClientError> {
        self.transport.post_json("datasets", Some(request)).await
    }

    pub async fn query(
        &self,
        request: QueryDatasetsRequest,
    ) -> Result<DatasetListResponse, ClientError> {
        self.transport
            .post_json("datasets/query", Some(request))
            .await
    }

    pub async fn versions(
        &self,
        dataset_name: &str,
        request: QueryDatasetVersionsRequest,
    ) -> Result<DatasetVersionListResponse, ClientError> {
        let mut url = self
            .transport
            .join(&format!("datasets/{dataset_name}/versions"));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(page) = request.page {
                pairs.append_pair("page", &page.to_string());
            }
            if let Some(per_page) = request.per_page {
                pairs.append_pair("per_page", &per_page.to_string());
            }
        }

        self.transport.get_json(url).await
    }

    pub async fn get_version(
        &self,
        dataset_name: &str,
        version: u32,
    ) -> Result<DatasetVersionResponse, ClientError> {
        self.transport
            .get_json(format!("datasets/{dataset_name}/versions/{version}"))
            .await
    }

    pub async fn get_latest_version(
        &self,
        dataset_name: &str,
    ) -> Result<DatasetVersionResponse, ClientError> {
        self.transport
            .get_json(format!("datasets/{dataset_name}/versions/latest"))
            .await
    }

    pub async fn download(
        &self,
        dataset_name: &str,
        version: u32,
    ) -> Result<DatasetDownloadResponse, ClientError> {
        self.transport
            .get_json(format!(
                "datasets/{dataset_name}/versions/{version}/download"
            ))
            .await
    }

    pub async fn stream_items(
        &self,
        dataset_name: &str,
        version: u32,
        request: StreamDatasetVersionItemsRequest,
    ) -> Result<DatasetVersionItemsPageResponse, ClientError> {
        let mut url = self
            .transport
            .join(&format!("datasets/{dataset_name}/versions/{version}/items"));
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(index) = request.index {
                pairs.append_pair("index", &index.to_string());
            }
            if let Some(limit) = request.limit {
                pairs.append_pair("limit", &limit.to_string());
            }
        }

        self.transport.get_json(url).await
    }
}
