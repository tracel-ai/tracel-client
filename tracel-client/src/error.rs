use reqwest::StatusCode;
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// A machine-readable error code, displayed the way the server spells it.
#[derive(Clone, Debug, Deserialize, strum::Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ApiErrorCode {
    ProjectAlreadyExists,
    UnsupportedSdkVersion,
    LimitReached,
    Dataset,
    DatasetVersion,
    Model,
    ModelVersion,
    ModelAlias,
    ModelVersionNotReady,
    ModelVersionDeleted,
    ModelVersionFailed,
    ModelVersionUploadIncomplete,
    MultipartUploadIncomplete,
    ModelVersionConflict,
    // ...
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
pub struct ApiErrorBody {
    #[serde(
        default = "unknown_api_error_code",
        deserialize_with = "read_a_null_code_as_unknown"
    )]
    pub code: ApiErrorCode,
    pub message: String,
}

fn unknown_api_error_code() -> ApiErrorCode {
    ApiErrorCode::Unknown
}

fn read_a_null_code_as_unknown<'de, D>(deserializer: D) -> Result<ApiErrorCode, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<ApiErrorCode>::deserialize(deserializer)?.unwrap_or(ApiErrorCode::Unknown))
}

impl Default for ApiErrorBody {
    fn default() -> Self {
        ApiErrorBody {
            code: ApiErrorCode::Unknown,
            message: "An unknown error occurred".to_string(),
        }
    }
}

impl Display for ApiErrorBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Bad session id")]
    BadSessionId,
    #[error("Resource not found")]
    NotFound,
    #[error("Resource not found: {0}")]
    NotFoundWithCode(ApiErrorCode),
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Api error {status}: {body}")]
    ApiError {
        status: StatusCode,
        body: ApiErrorBody,
    },
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("Unknown Error: {0}")]
    UnknownError(String),
}

impl ClientError {
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            ClientError::NotFound | ClientError::NotFoundWithCode(_)
        )
    }

    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            ClientError::ApiError { status, .. } if *status == StatusCode::CONFLICT
        )
    }

    pub fn code(&self) -> Option<ApiErrorCode> {
        match self {
            ClientError::ApiError { body, .. } => Some(body.code.clone()),
            ClientError::NotFoundWithCode(code) => Some(code.clone()),
            _ => None,
        }
    }

    pub fn is_login_error(&self) -> bool {
        matches!(self, ClientError::Unauthorized)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        match error.status() {
            Some(status) => ClientError::ApiError {
                status,
                body: ApiErrorBody {
                    code: ApiErrorCode::Unknown,
                    message: error.to_string(),
                },
            },
            None => ClientError::UnknownError(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_body_with_a_null_code_reads_as_unknown_and_keeps_the_message() {
        let body: ApiErrorBody =
            serde_json::from_str(r#"{"message": "Unauthorized access", "code": null}"#).unwrap();

        assert!(matches!(body.code, ApiErrorCode::Unknown));
        assert_eq!(body.message, "Unauthorized access");
    }

    #[test]
    fn api_error_body_without_a_code_reads_as_unknown_and_keeps_the_message() {
        let body: ApiErrorBody =
            serde_json::from_str(r#"{"message": "Unauthorized access"}"#).unwrap();

        assert!(matches!(body.code, ApiErrorCode::Unknown));
        assert_eq!(body.message, "Unauthorized access");
    }

    #[test]
    fn api_error_body_with_a_known_code_reads_as_its_variant() {
        let body: ApiErrorBody = serde_json::from_str(
            r#"{"message": "Project already exists", "code": "PROJECT_ALREADY_EXISTS"}"#,
        )
        .unwrap();

        assert!(matches!(body.code, ApiErrorCode::ProjectAlreadyExists));
    }

    #[test]
    fn api_error_body_with_an_unrecognised_code_reads_as_unknown_and_keeps_the_message() {
        let body: ApiErrorBody = serde_json::from_str(
            r#"{"message": "Something went wrong", "code": "A_CODE_THIS_CLIENT_DOES_NOT_KNOW"}"#,
        )
        .unwrap();

        assert!(matches!(body.code, ApiErrorCode::Unknown));
        assert_eq!(body.message, "Something went wrong");
    }
}
