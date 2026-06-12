use serde::Serialize;
use std::str::FromStr;

/// Credentials to connect to the Tracel server
#[derive(Serialize, Debug, Clone)]
pub struct TracelCredentials {
    api_key: String,
}

impl TracelCredentials {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Creates a new instance of `TracelCredentials` from environment variables.
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let api_key = std::env::var("TRACEL_API_KEY")?;
        Ok(Self::new(api_key))
    }
}

impl FromStr for TracelCredentials {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("API key cannot be empty".to_string())
        } else {
            Ok(Self::new(s))
        }
    }
}
