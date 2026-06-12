pub mod annotation;
pub mod dataset;
pub mod experiment;
pub mod model;
pub mod system;

use reqwest::Url;

use crate::transport::ApiTransport;

#[derive(Debug, Clone)]
pub struct StationClient {
    transport: ApiTransport,
}

impl StationClient {
    pub fn from_url(base_url: Url) -> Self {
        Self {
            transport: ApiTransport::new(base_url),
        }
    }

    pub fn experiments(&self) -> experiment::ExperimentClient<'_> {
        experiment::ExperimentClient::new(&self.transport)
    }

    pub fn models(&self) -> model::ModelClient<'_> {
        model::ModelClient::new(&self.transport)
    }

    pub fn datasets(&self) -> dataset::DatasetClient<'_> {
        dataset::DatasetClient::new(&self.transport)
    }

    pub fn annotation_sets(&self) -> annotation::AnnotationClient<'_> {
        annotation::AnnotationClient::new(&self.transport)
    }

    pub fn system(&self) -> system::SystemClient<'_> {
        system::SystemClient::new(&self.transport)
    }
}
