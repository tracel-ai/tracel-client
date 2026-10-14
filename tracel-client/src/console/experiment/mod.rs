pub mod request;
pub mod response;

mod body;

use std::collections::HashMap;

use reqwest::Url;
use serde_json::Value;

use crate::{
    ClientError, WebSocketClient,
    console::Client,
    console::experiment::{
        body::CreateExperimentSchema,
        request::{
            ExperimentLogQueryRequest, ListExperimentsQuery, MetricAggregatedQuery,
            MetricSummaryQuery,
        },
        response::{
            ActivityTreeResponse, ExperimentDetailsResponse, ExperimentLogQueryResponse,
            ExperimentResponse, ListExperimentsResponse, MetricMetadataResponse, MetricResponse,
            MetricSummaryResponse,
        },
    },
    websocket::WebSocketError,
};

impl Client {
    /// List project experiments using the v1 API.
    pub fn get_project_experiments(
        &self,
        owner_name: &str,
        project_name: &str,
        query: ListExperimentsQuery,
    ) -> Result<ListExperimentsResponse, ClientError> {
        let mut url = self
            .transport
            .join(&format!("projects/{owner_name}/{project_name}/experiments"));
        append_list_query(&query, &mut url);

        self.transport.get_json(url)
    }

    /// Get an experiment by number using the v1 API.
    pub fn get_experiment(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
    ) -> Result<ExperimentDetailsResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{owner_name}/{project_name}/experiments/{exp_num}"
        ))
    }

    /// Get the latest project experiment, or `None`, using the v1 API.
    pub fn get_project_latest_experiment(
        &self,
        owner_name: &str,
        project_name: &str,
    ) -> Result<Option<ExperimentDetailsResponse>, ClientError> {
        self.transport.get_json(format!(
            "projects/{owner_name}/{project_name}/experiments/latest"
        ))
    }

    /// Get one metric series, or `None` for HTTP 204, using the v1 API.
    pub fn get_metrics(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
        query: MetricAggregatedQuery,
    ) -> Result<Option<MetricResponse>, ClientError> {
        let mut url = self.transport.join(&format!(
            "projects/{owner_name}/{project_name}/experiments/{exp_num}/metrics"
        ));
        append_metric_query(&query, &mut url);

        self.transport.get_optional_json(url)
    }

    /// Get experiment metric metadata using the v1 API.
    pub fn get_metric_metadata(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
    ) -> Result<MetricMetadataResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{owner_name}/{project_name}/experiments/{exp_num}/metrics/metadata"
        ))
    }

    /// Get a metric summary, or `None` for HTTP 204, using the v1 API.
    pub fn get_metric_summary(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
        query: MetricSummaryQuery,
    ) -> Result<Option<MetricSummaryResponse>, ClientError> {
        let mut url = self.transport.join(&format!(
            "projects/{owner_name}/{project_name}/experiments/{exp_num}/metrics/summary"
        ));
        url.query_pairs_mut().append_pair("metric", &query.metric);

        self.transport.get_optional_json(url)
    }

    /// Get the experiment activity tree using the v1 API.
    pub fn get_activities(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
    ) -> Result<ActivityTreeResponse, ClientError> {
        self.transport.get_json(format!(
            "projects/{owner_name}/{project_name}/experiments/{exp_num}/activities"
        ))
    }

    /// Query experiment logs using the v1 API.
    pub fn query_experiment_logs(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
        request: ExperimentLogQueryRequest,
    ) -> Result<ExperimentLogQueryResponse, ClientError> {
        self.transport.post_json(
            format!("projects/{owner_name}/{project_name}/experiments/{exp_num}/logs/query"),
            Some(request),
        )
    }

    /// Formats a WebSocket URL for the given experiment.
    fn format_websocket_url(&self, owner_name: &str, project_name: &str, exp_num: i32) -> String {
        let path: &str = &format!("projects/{owner_name}/{project_name}/experiments/{exp_num}/ws");
        let mut url = self.transport.join(path);

        url.set_scheme(if self.transport.base_url().scheme() == "https" {
            "wss"
        } else {
            "ws"
        })
        .expect("Should be able to set ws scheme");

        url.to_string()
    }

    /// Create a new experiment for the given project.
    ///
    /// The client must be logged in before calling this method.
    pub fn create_experiment(
        &self,
        owner_name: &str,
        project_name: &str,
        name: Option<String>,
        description: Option<String>,
        attributes: HashMap<String, Value>,
    ) -> Result<ExperimentResponse, ClientError> {
        let path: &str = &format!("projects/{owner_name}/{project_name}/experiments");
        let url = self.transport.join(path);

        // Create a new experiment
        let experiment_response = self.transport.post_json(
            url,
            Some(CreateExperimentSchema {
                name,
                description,
                attributes,
            }),
        )?;

        Ok(experiment_response)
    }

    pub fn create_experiment_run_websocket(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
    ) -> Result<WebSocketClient, WebSocketError> {
        let mut ws_client = WebSocketClient::new();

        let ws_endpoint = self.format_websocket_url(owner_name, project_name, exp_num);

        ws_client
            .connect(&ws_endpoint, self.transport.auth())
            .map_err(|e| WebSocketError::ConnectionError(e.to_string()))?;

        Ok(ws_client)
    }

    /// Cancel an experiment.
    ///
    /// The client must be logged in before calling this method.
    pub fn cancel_experiment(
        &self,
        owner_name: &str,
        project_name: &str,
        exp_num: i32,
    ) -> Result<(), ClientError> {
        let path = &format!("projects/{owner_name}/{project_name}/experiments/{exp_num}/cancel");
        let url = self.transport.join(path);

        self.transport.post(url, None::<()>)
    }
}

fn append_list_query(query: &ListExperimentsQuery, url: &mut Url) {
    if let Some(page) = query.page {
        url.query_pairs_mut().append_pair("page", &page.to_string());
    }
    if let Some(limit) = query.limit {
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string());
    }
    for sort in &query.sort {
        url.query_pairs_mut().append_pair("sort", sort);
    }
}

fn append_metric_query(query: &MetricAggregatedQuery, url: &mut Url) {
    url.query_pairs_mut()
        .append_pair("metric", &query.metric)
        .append_pair("max_points", &query.max_points.to_string())
        .append_pair(
            "downsampling_factor",
            &query.downsampling_factor.to_string(),
        );
}
