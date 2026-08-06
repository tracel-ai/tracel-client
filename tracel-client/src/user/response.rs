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

#[cfg(test)]
mod tests {
    use super::*;

    /// A dead session is reported as a `null` body on an otherwise successful response.
    #[test]
    fn current_user_is_absent_when_the_session_is_dead() {
        let user: Option<UserResponseSchema> = serde_json::from_str("null").unwrap();

        assert!(user.is_none());
    }

    #[test]
    fn current_user_matches_backend_contract() {
        let user: Option<UserResponseSchema> = serde_json::from_str(
            r#"{"id":7,"username":"ada","email":"ada@example.com","namespace":"ada"}"#,
        )
        .unwrap();

        let user = user.expect("a live session should carry a user");
        assert_eq!(user._id, 7);
        assert_eq!(user.username, "ada");
        assert_eq!(user.email, "ada@example.com");
        assert_eq!(user.namespace, "ada");
    }
}
