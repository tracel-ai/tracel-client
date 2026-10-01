use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct UserResponseSchema {
    #[serde(rename = "id")]
    pub _id: i32,
    pub username: String,
    pub email: Option<String>,
    pub namespace: String,
    /// The credential the user was read with.
    pub credential: CredentialSchema,
}

/// The credential a request was authenticated with.
#[derive(Deserialize, Clone, Debug)]
pub struct CredentialSchema {
    pub kind: CredentialKind,
    /// When it stops being accepted, as RFC 3339: an API key's expiry, or an
    /// app session's access token expiry. `None` for a key without expiry.
    pub expires_at: Option<String>,
}

/// The kinds of credential the server tells apart.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CredentialKind {
    BrowserSession,
    AppSession,
    ApiKey,
    #[serde(other)]
    Unknown,
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
