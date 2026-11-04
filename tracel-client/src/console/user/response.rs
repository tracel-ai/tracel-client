use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct UserResponseSchema {
    #[serde(rename = "id")]
    pub _id: i32,
    pub username: String,
    pub email: Option<String>,
    pub namespace: String,
    /// The profile image selected in the console, when one is available.
    #[serde(default)]
    pub profile_picture_url: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_response_keeps_the_console_avatar_and_accepts_its_absence() {
        let mut response = serde_json::json!({
            "id": 7,
            "username": "ada",
            "email": "ada@example.com",
            "namespace": "ada",
            "credential": { "kind": "app_session", "expires_at": null }
        });
        let user: UserResponseSchema = serde_json::from_value(response.clone()).unwrap();
        assert!(user.profile_picture_url.is_none());
        response["profile_picture_url"] = serde_json::json!("https://example.com/ada.png");
        let user: UserResponseSchema = serde_json::from_value(response.clone()).unwrap();
        assert_eq!(
            user.profile_picture_url.as_deref(),
            Some("https://example.com/ada.png")
        );
        response["profile_picture_url"] = serde_json::Value::Null;
        let user: UserResponseSchema = serde_json::from_value(response).unwrap();
        assert!(user.profile_picture_url.is_none());
    }
}
