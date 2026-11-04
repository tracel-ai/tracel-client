use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ComputeProviderQueueJobRequest {
    pub compute_provider_group_name: String,
    pub digest: String,
    pub command: String,
}
