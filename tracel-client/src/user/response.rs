use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct UserResponseSchema {
    #[serde(rename = "id")]
    pub _id: i32,
    pub username: String,
    pub email: String,
    pub namespace: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetUserOrganizationsResponse {
    pub organizations: Vec<OrganizationResponse>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OrganizationResponse {
    pub name: String,
    pub namespace: String,
}
