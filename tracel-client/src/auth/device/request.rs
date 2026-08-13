use serde::Serialize;

/// Form body of `POST auth/device/code`.
#[derive(Serialize, Clone, Debug)]
pub struct DeviceCodeRequest<'a> {
    pub client_id: &'a str,
}

/// Form body of `POST auth/token`.
#[derive(Serialize, Clone, Debug)]
pub struct DeviceTokenRequest<'a> {
    pub grant_type: &'a str,
    pub device_code: &'a str,
    pub client_id: &'a str,
}
