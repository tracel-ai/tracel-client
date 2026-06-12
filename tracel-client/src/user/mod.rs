pub mod response;

use reqwest::header::SET_COOKIE;

use crate::{
    Client, ClientError,
    tracel::TracelCredentials,
    transport::ResponseExt,
    user::response::{GetUserOrganizationsResponse, UserResponseSchema},
};

impl Client {
    /// Log in to the Tracel server with the given credentials.
    pub fn login(&self, credentials: &TracelCredentials) -> Result<String, ClientError> {
        let form = self
            .transport
            .request(reqwest::Method::POST, "login/api-key")
            .form::<TracelCredentials>(credentials);

        tracing::debug!("Requesting login form: {form:?}");

        let res = form.send()?.map_to_tracel_err()?;

        let cookie_header = res.headers().get(SET_COOKIE);
        if let Some(cookie) = cookie_header {
            let cookie_str = cookie
                .to_str()
                .expect("Session cookie should be able to convert to str");
            Ok(cookie_str.to_string())
        } else {
            Err(ClientError::BadSessionId)
        }
    }

    pub fn get_current_user(&self) -> Result<UserResponseSchema, ClientError> {
        let url = self.transport.join("user");
        self.transport.get_json(url)
    }

    pub fn get_user_organizations(&self) -> Result<GetUserOrganizationsResponse, ClientError> {
        let url = self.transport.join("user/organizations");

        self.transport.get_json(url)
    }
}
