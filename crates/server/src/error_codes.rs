use axum::http::StatusCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    TaskNotFound,
    WorkerNotFound,
    InvalidToken,
    InvalidStatus,
    InvalidRequest,
    DatabaseError,
    SerializationError,
    InternalError,
    WorkerRegistrationFailed,
    TaskCreationFailed,
    FeedbackRejected,
    OperationTimeout,
    ServiceUnavailable,
}

impl ErrorCode {
    pub fn http_status(&self) -> StatusCode {
        match self {
            ErrorCode::TaskNotFound | ErrorCode::WorkerNotFound => StatusCode::NOT_FOUND,
            ErrorCode::InvalidToken => StatusCode::UNAUTHORIZED,
            ErrorCode::InvalidStatus | ErrorCode::InvalidRequest | ErrorCode::FeedbackRejected => {
                StatusCode::BAD_REQUEST
            }
            ErrorCode::DatabaseError
            | ErrorCode::SerializationError
            | ErrorCode::InternalError
            | ErrorCode::WorkerRegistrationFailed
            | ErrorCode::TaskCreationFailed => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::OperationTimeout => StatusCode::REQUEST_TIMEOUT,
            ErrorCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::TaskNotFound => "TASK_NOT_FOUND",
            ErrorCode::WorkerNotFound => "WORKER_NOT_FOUND",
            ErrorCode::InvalidToken => "INVALID_TOKEN",
            ErrorCode::InvalidStatus => "INVALID_STATUS",
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::DatabaseError => "DATABASE_ERROR",
            ErrorCode::SerializationError => "SERIALIZATION_ERROR",
            ErrorCode::InternalError => "INTERNAL_ERROR",
            ErrorCode::WorkerRegistrationFailed => "WORKER_REGISTRATION_FAILED",
            ErrorCode::TaskCreationFailed => "TASK_CREATION_FAILED",
            ErrorCode::FeedbackRejected => "FEEDBACK_REJECTED",
            ErrorCode::OperationTimeout => "OPERATION_TIMEOUT",
            ErrorCode::ServiceUnavailable => "SERVICE_UNAVAILABLE",
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
