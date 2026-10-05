//! Turning [`TracelCredentials`] into the credential requests carry.
//!
//! An API key is sent as a bearer token; a session token names a cookie session.

use crate::console::credentials::TracelCredentials;
use crate::error::ClientError;
use crate::transport::Auth;

/// The credential requests carry for `credentials`.
///
/// It does not prove the credential is live; the caller does that.
pub fn authenticate(credentials: &TracelCredentials) -> Result<Auth, ClientError> {
    match credentials {
        TracelCredentials::ApiKey(api_key) => Auth::bearer(api_key),
        TracelCredentials::SessionToken(session_token) => {
            Ok(Auth::session_token(session_token.as_str()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::credentials::SessionToken;

    const KEY: &str = "tcl_key_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefg012345";

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
    fn a_session_token_is_sent_as_the_session_cookie() {
        let auth = authenticate(&SessionToken::new("abc").into()).unwrap();

        assert!(matches!(auth, Auth::SessionCookie(cookie) if cookie == "id=abc"));
    }
}
