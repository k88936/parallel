use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use crate::protocol::*;
use crate::server::db::entity::{human_sessions, tasks};
use crate::server::state::AppState;

pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    tracing::info!("Creating session for task: {}", payload.task_id);

    let task = tasks::Entity::find_by_id(payload.task_id)
        .one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let worker_id = task.claimed_by.ok_or(StatusCode::PRECONDITION_FAILED)?;

    let session_id = Uuid::new_v4();
    let permissions = payload.permissions.unwrap_or_default();
    let permissions_json = serde_json::to_string(&permissions).map_err(|e| {
        tracing::error!("Failed to serialize permissions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let session = human_sessions::ActiveModel {
        session_id: Set(session_id),
        task_id: Set(payload.task_id),
        worker_id: Set(worker_id),
        attached_at: Set(Utc::now()),
        permissions_json: Set(permissions_json),
    };

    session.insert(&state.db).await.map_err(|e| {
        tracing::error!("Failed to create session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Session {} created for task {}", session_id, payload.task_id);

    Ok(Json(CreateSessionResponse { session_id }))
}
