use serde::Deserialize;

/// An inference group as returned by the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct InferenceGroupResponse {
    /// UUID of the inference group.
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}
