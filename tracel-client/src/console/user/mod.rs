pub mod response;

use crate::{
    ClientError,
    console::Client,
    console::user::response::{GetUserOrganizationsResponse, UserResponseSchema},
};

impl Client {
    /// Fetches the authenticated user.
    ///
    /// Returns `Ok(None)` when the session is missing or expired because the server reports that
    /// state as a successful response with a `null` body.
    ///
    /// [`Client::user`] returns the copy taken on connect, without a request.
    pub fn get_current_user(&self) -> Result<Option<UserResponseSchema>, ClientError> {
        let url = self.transport.join("user");
        self.transport.get_json(url)
    }

    pub fn get_user_organizations(&self) -> Result<GetUserOrganizationsResponse, ClientError> {
        let url = self.transport.join("user/organizations");

        self.transport.get_json(url)
    }
}
