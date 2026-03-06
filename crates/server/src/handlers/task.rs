use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use parallel_protocol::*;

use crate::errors::ServerError;
use crate::services::{Coordinator, TaskService};
use crate::state::AppState;

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, StatusCode> {
    let base_branch = payload.base_branch.unwrap_or_else(|| "main".to_string());
    let target_branch = payload.target_branch.unwrap_or_else(|| {
        format!("task/{}", Uuid::new_v4())
    });
    let priority = payload.priority.unwrap_or_default();

    let task_service = TaskService::new(state.db.clone());

    match task_service
        .create(
            payload.repo_url,
            payload.description,
            base_branch,
            target_branch,
            priority,
            payload.ssh_key,
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

    let task_service = TaskService::new(state.db.clone());

    match task_service.list(status, limit, offset).await {
        Ok(tasks) => {
            let total = task_service.count(status).await.unwrap_or(0);
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
    let task_service = TaskService::new(state.db.clone());

    match task_service.get(&task_id).await {
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
    let task_service = TaskService::new(state.db.clone());
    let coordinator = Coordinator::new(state.db.clone());

    match task_service.get(&task_id).await {
        Ok(task) => {
            if let Some(worker_id) = task.claimed_by {
                let _ = coordinator
                    .queue_cancellation(worker_id, task_id, "Cancelled by user".to_string())
                    .await;
            }

            match task_service.update_status(&task_id, TaskStatus::Cancelled).await {
                Ok(()) => Ok(StatusCode::NO_CONTENT),
                Err(e) => {
                    tracing::error!("Failed to cancel task: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get task for cancellation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn submit_feedback(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<SubmitFeedbackRequest>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Feedback submitted for task {}", task_id);

    let task_service = TaskService::new(state.db.clone());
    let coordinator = Coordinator::new(state.db.clone());

    let feedback = HumanFeedback {
        provided_at: chrono::Utc::now(),
        feedback_type: payload.feedback_type,
        message: payload.message,
    };

    match task_service.get(&task_id).await {
        Ok(task) => {
            match task.claimed_by {
                Some(worker_id) => {
                    match coordinator.queue_feedback(worker_id, task_id, feedback.clone()).await {
                        Ok(()) => {
                            if matches!(feedback.feedback_type, parallel_protocol::FeedbackType::RequestChanges) {
                                if let Err(e) = task_service.update_status(&task_id, TaskStatus::PendingRework).await {
                                    tracing::error!("Failed to update task {} status to PendingRework: {}", task_id, e);
                                }
                            }
                            Ok(StatusCode::NO_CONTENT)
                        }
                        Err(e) => {
                            tracing::error!("Failed to submit feedback for task {}: {}", task_id, e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                }
                None => {
                    tracing::warn!("Feedback rejected for unclaimed task {}", task_id);
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        }
        Err(ServerError::TaskNotFound(_)) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get task for feedback: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_review_data(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Option<ReviewData>>, StatusCode> {
    let task_service = TaskService::new(state.db.clone());

    match task_service.get_review_data(&task_id).await {
        Ok(review_data) => Ok(Json(review_data)),
        Err(e) => {
            tracing::error!("Failed to get review data for task {}: {}", task_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskStatusRequest>,
) -> Result<StatusCode, StatusCode> {
    let task_service = TaskService::new(state.db.clone());

    match task_service.complete_iteration(&task_id, payload.status).await {
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
