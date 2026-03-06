use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::protocol::*;
use crate::server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, StatusCode> {
    let base_branch = payload.base_branch.unwrap_or_else(|| "main".to_string());
    let target_branch = payload.target_branch.unwrap_or_else(|| {
        format!("task/{}", Uuid::new_v4())
    });
    let priority = payload.priority.unwrap_or_default();

    match state.scheduler
        .create_task(
            payload.repo_url,
            payload.description,
            base_branch,
            target_branch,
            priority,
        )
        .await
    {
        Ok(task_id) => Ok(Json(CreateTaskResponse { task_id })),
        Err(e) => {
            tracing::error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<TaskListResponse>, StatusCode> {
    let limit = query.limit.map(|l| l as u64);
    let offset = query.offset.map(|o| o as u64);
    let status = query.status;

    match state.scheduler
        .list_tasks(status, limit, offset)
        .await
    {
        Ok(tasks) => {
            let total = state.scheduler
                .count_tasks(status)
                .await
                .unwrap_or(0);

            Ok(Json(TaskListResponse { tasks, total }))
        }
        Err(e) => {
            tracing::error!("Failed to list tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Task>, StatusCode> {
    match state.scheduler.get_task(&task_id).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => {
            tracing::error!("Failed to get task: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn cancel_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.scheduler.cancel_task(&task_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            tracing::error!("Failed to cancel task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn submit_feedback(
    State(_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(_payload): Json<SubmitFeedbackRequest>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Feedback submitted for task {}", task_id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskStatusRequest>,
) -> Result<StatusCode, StatusCode> {
    match state
        .scheduler
        .complete_iteration(&task_id, payload.status)
        .await
    {
        Ok(()) => {
            tracing::info!(
                "Task {} status updated to {:?}",
                task_id,
                payload.status
            );
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            tracing::error!("Failed to update task status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}