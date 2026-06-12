use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub(crate) struct CreateExperimentSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: HashMap<String, Value>,
}
