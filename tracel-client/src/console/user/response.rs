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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_user_read_with_an_api_key_carries_the_key_and_its_expiry() {
        let user: UserResponseSchema = serde_json::from_str(
            r#"{"id": 1, "username": "alice", "namespace": "alice", "profile_picture_url": null,
                "credential": {"kind": "api_key", "expires_at": "2026-12-30T12:00:00.000Z"}}"#,
        )
        .unwrap();

        assert_eq!(user.credential.kind, CredentialKind::ApiKey);
        assert_eq!(
            user.credential.expires_at.as_deref(),
            Some("2026-12-30T12:00:00.000Z")
        );
        assert!(user.email.is_none());
    }

    #[test]
    fn a_credential_kind_this_client_does_not_know_reads_as_unknown() {
        let credential: CredentialSchema =
            serde_json::from_str(r#"{"kind": "job_token", "expires_at": null}"#).unwrap();

        assert_eq!(credential.kind, CredentialKind::Unknown);
    }
}
