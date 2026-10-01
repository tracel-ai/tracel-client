//! Turning [`TracelCredentials`] into the credential requests carry.
//!
//! Every kind is sent as `Authorization: Bearer`; an app session renews its
//! access token as requests need it.

use crate::console::credentials::TracelCredentials;
use crate::error::ClientError;
use crate::transport::Auth;

/// The credential requests carry for `credentials`.
///
/// It does not prove the credential is live; the caller does that.
pub fn authenticate(credentials: &TracelCredentials) -> Result<Auth, ClientError> {
    match credentials {
        TracelCredentials::ApiKey(api_key) => Auth::bearer(api_key),
        TracelCredentials::AccessToken(access_token) => Auth::bearer(access_token.as_str()),
        TracelCredentials::AppSession(app_session) => Ok(Auth::AppSession(app_session.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::credentials::AccessToken;

    const KEY: &str = "tcl_key_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefg012345";
    const ACCESS_TOKEN: &str = "tcl_at_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefg012345";

    #[test]
    fn an_api_key_is_sent_as_a_sensitive_bearer_token() {
        let auth = authenticate(&TracelCredentials::api_key(KEY)).unwrap();

        let Auth::Bearer(value) = auth else {
            panic!("an API key should be sent as a bearer token");
        };
        assert_eq!(value.to_str().unwrap(), format!("Bearer {KEY}"));
        assert!(value.is_sensitive());
    }

    #[test]
    fn a_bearer_credential_does_not_print_its_token() {
        let auth = authenticate(&TracelCredentials::api_key(KEY)).unwrap();

        assert!(!format!("{auth:?}").contains(KEY));
    }

    #[test]
    fn an_access_token_is_sent_as_a_sensitive_bearer_token() {
        let auth = authenticate(&AccessToken::new(ACCESS_TOKEN).into()).unwrap();

        let Auth::Bearer(value) = auth else {
            panic!("an access token should be sent as a bearer token");
        };
        assert_eq!(value.to_str().unwrap(), format!("Bearer {ACCESS_TOKEN}"));
        assert!(value.is_sensitive());
    }
}
