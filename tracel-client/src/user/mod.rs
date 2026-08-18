pub mod response;

use crate::{
    Client, ClientError,
    user::response::{GetUserOrganizationsResponse, UserResponseSchema},
};

impl Client {
    /// Fetches the authenticated user.
    ///
    /// [`Client::user`] returns the copy taken on connect, without a request.
    pub fn get_current_user(&self) -> Result<UserResponseSchema, ClientError> {
        let url = self.transport.join("user");

        // The endpoint answers 200 with a `null` body rather than 401 when the
        // session is missing or expired.
        self.transport
            .get_json::<Option<UserResponseSchema>>(url)?
            .ok_or(ClientError::Unauthorized)
    }

    pub fn get_user_organizations(&self) -> Result<GetUserOrganizationsResponse, ClientError> {
        let url = self.transport.join("user/organizations");

        self.transport.get_json(url)
    }
}
