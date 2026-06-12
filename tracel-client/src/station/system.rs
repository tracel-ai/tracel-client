use crate::{ClientError, transport::ApiTransport};

pub struct SystemClient<'a> {
    transport: &'a ApiTransport,
}

impl<'a> SystemClient<'a> {
    pub(crate) fn new(transport: &'a ApiTransport) -> Self {
        Self { transport }
    }

    pub fn health(&self) -> Result<(), ClientError> {
        let url = self
            .transport
            .base_url()
            .join("health")
            .expect("Should be able to join health url");

        self.transport.get(url)
    }
}
