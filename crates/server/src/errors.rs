use axum::http::StatusCode;
use uuid::Uuid;

#[derive(Debug)]
pub enum ServerError {
    TaskNotFound(Uuid),
    WorkerNotFound(Uuid),
    ProjectNotFound(Uuid),
    InvalidToken,
    InvalidStatus(String),
    DatabaseError(String),
    SerializationError(String),
    InternalError(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            ServerError::WorkerNotFound(id) => write!(f, "Worker not found: {}", id),
            ServerError::ProjectNotFound(id) => write!(f, "Project not found: {}", id),
            ServerError::InvalidToken => write!(f, "Invalid token"),
            ServerError::InvalidStatus(s) => write!(f, "Invalid status: {}", s),
            ServerError::DatabaseError(s) => write!(f, "Database error: {}", s),
            ServerError::SerializationError(s) => write!(f, "Serialization error: {}", s),
            ServerError::InternalError(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl std::error::Error for ServerError {}

impl From<ServerError> for StatusCode {
    fn from(err: ServerError) -> StatusCode {
        match err {
            ServerError::TaskNotFound(_) => StatusCode::NOT_FOUND,
            ServerError::WorkerNotFound(_) => StatusCode::NOT_FOUND,
            ServerError::ProjectNotFound(_) => StatusCode::NOT_FOUND,
            ServerError::InvalidToken => StatusCode::UNAUTHORIZED,
            ServerError::InvalidStatus(_) => StatusCode::BAD_REQUEST,
            ServerError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<sea_orm::DbErr> for ServerError {
    fn from(err: sea_orm::DbErr) -> Self {
        ServerError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::SerializationError(err.to_string())
    }
}

pub type ServerResult<T> = Result<T, ServerError>;
