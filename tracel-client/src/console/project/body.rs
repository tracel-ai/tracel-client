use serde::Serialize;

use super::request::Visibility;

#[derive(Serialize, Clone, Debug)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Visibility,
}
