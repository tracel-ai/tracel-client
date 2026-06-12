pub mod request;
pub mod response;
pub mod websocket;

use std::collections::HashMap;

use serde_json::Value;

use crate::{
    Client, ClientError, WebSocketClient,
    experiment::{request::CreateExperimentSchema, response::ExperimentResponse},
    websocket::WebSocketError,
};

impl Client {
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
