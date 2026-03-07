use crate::error_codes::ErrorCode;
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error: ErrorBody {
                code,
                message: message.into(),
                details: None,
                metadata: None,
            },
            correlation_id: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.error.details = Some(details.into());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let metadata = self.error.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.into(), value);
        self
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = self.error.code.http_status();
        (status, Json(self)).into_response()
    }
}

impl From<crate::errors::ServerError> for ErrorResponse {
    fn from(err: crate::errors::ServerError) -> Self {
        use crate::errors::ServerError;
        match err {
            ServerError::TaskNotFound(id) => ErrorResponse::new(
                ErrorCode::TaskNotFound,
                format!("Task with ID {} not found", id),
            )
            .with_metadata("task_id", serde_json::json!(id)),
            ServerError::WorkerNotFound(id) => ErrorResponse::new(
                ErrorCode::WorkerNotFound,
                format!("Worker with ID {} not found", id),
            )
            .with_metadata("worker_id", serde_json::json!(id)),
            ServerError::InvalidStatus(s) => {
                ErrorResponse::new(ErrorCode::InvalidStatus, format!("Invalid status: {}", s))
            }
            ServerError::DatabaseError(s) => {
                ErrorResponse::new(ErrorCode::DatabaseError, "Database operation failed")
                    .with_details(s)
            }
            ServerError::SerializationError(s) => {
                ErrorResponse::new(ErrorCode::SerializationError, "Serialization failed")
                    .with_details(s)
            }
            ServerError::InternalError(s) => {
                ErrorResponse::new(ErrorCode::InternalError, "Internal server error")
                    .with_details(s)
            }
        }
    }
}

pub type ApiResult<T> = Result<T, ErrorResponse>;

pub fn map_anyhow_to_api_error(
    err: anyhow::Error,
    code: ErrorCode,
    message: impl Into<String>,
) -> ErrorResponse {
    ErrorResponse::new(code, message).with_details(err.to_string())
}
