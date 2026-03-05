use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::protocol::*;

pub async fn create_session(
    State(_state): State<crate::server::state::AppState>,
    Json(_payload): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    tracing::info!("Session creation requested (not yet implemented)");
    Err(StatusCode::NOT_IMPLEMENTED)
}
