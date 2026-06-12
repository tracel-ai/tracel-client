use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
pub struct ExperimentResponse {
    pub experiment_num: i32,
    pub name: Option<String>,
    pub attributes: HashMap<String, Value>,
}
