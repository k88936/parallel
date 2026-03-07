use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use parallel_protocol::*;

use crate::api_error::{ApiResult, ErrorResponse};
use crate::error_codes::ErrorCode;
use crate::errors::ServerError;
use crate::services::traits::TaskListParams;
use crate::state::AppState;

pub async fn create_task(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<CreateTaskRequest>,
) -> ApiResult<Json<CreateTaskResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let base_branch = payload.base_branch.unwrap_or_else(|| "main".to_string());
    let target_branch = payload
        .target_branch
        .unwrap_or_else(|| format!("task/{}", Uuid::new_v4()));
    let priority = payload.priority.unwrap_or_default();
    let max_execution_time = payload.max_execution_time.unwrap_or(3600);

    let (repo_url, ssh_key) = if let Some(project_id) = payload.project_id {
        let project = state.project_service.get(&project_id).await.map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                project_id = %project_id,
                error = %e,
                "Failed to get project"
            );
            ErrorResponse::new(ErrorCode::InternalError, "Failed to get project")
                .with_details(e.to_string())
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

        let repo_url = if let Some(ref repo_ref) = payload.repo_ref {
            project
                .repos
                .iter()
                .find(|r| &r.name == repo_ref)
                .map(|r| r.url.clone())
                .ok_or_else(|| {
                    ErrorResponse::new(ErrorCode::InternalError, format!("Repo '{}' not found in project", repo_ref))
                        .with_correlation_id(correlation_id.unwrap_or_default())
                })?
        } else if let Some(ref url) = payload.repo_url {
            url.clone()
        } else {
            return Err(ErrorResponse::new(
                ErrorCode::TaskCreationFailed,
                "Either repo_url or repo_ref must be provided",
            )
            .with_correlation_id(correlation_id.unwrap_or_default()));
        };

        let ssh_key = if let Some(ref key_ref) = payload.ssh_key_ref {
            project
                .ssh_keys
                .iter()
                .find(|k| &k.name == key_ref)
                .map(|k| k.key.clone())
                .ok_or_else(|| {
                    ErrorResponse::new(ErrorCode::InternalError, format!("SSH key '{}' not found in project", key_ref))
                        .with_correlation_id(correlation_id.unwrap_or_default())
                })?
        } else if let Some(ref key) = payload.ssh_key {
            key.clone()
        } else {
            return Err(ErrorResponse::new(
                ErrorCode::TaskCreationFailed,
                "Either ssh_key or ssh_key_ref must be provided",
            )
            .with_correlation_id(correlation_id.unwrap_or_default()));
        };

        (repo_url, ssh_key)
    } else {
        let repo_url = payload.repo_url.ok_or_else(|| {
            ErrorResponse::new(ErrorCode::TaskCreationFailed, "repo_url is required when project_id is not provided")
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;
        let ssh_key = payload.ssh_key.ok_or_else(|| {
            ErrorResponse::new(ErrorCode::TaskCreationFailed, "ssh_key is required when project_id is not provided")
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;
        (repo_url, ssh_key)
    };

    tracing::info!(
        correlation_id = ?correlation_id,
        repo_url = %repo_url,
        title = %payload.title,
        "Creating task"
    );

    let task_id = state
        .task_service
        .create(
            payload.title,
            repo_url,
            payload.description,
            base_branch,
            target_branch,
            priority,
            ssh_key,
            max_execution_time,
            payload.project_id,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                error = %e,
                "Failed to create task"
            );
            ErrorResponse::new(ErrorCode::TaskCreationFailed, "Failed to create task")
                .with_details(e.to_string())
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        task_id = %task_id,
        "Task created successfully"
    );

    Ok(Json(CreateTaskResponse { task_id }))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Query(query): Query<ListTasksQuery>,
) -> ApiResult<Json<TaskListResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let params = TaskListParams {
        status: query.status,
        priority: query.priority,
        repo_url: query.repo_url,
        worker_id: query.worker_id,
        search: query.search,
        created_after: query.created_after,
        created_before: query.created_before,
        sort_by: query.sort_by,
        sort_direction: query.sort_direction,
        cursor: query.cursor,
        limit: query.limit.map(|l| l as u64),
        offset: query.offset.map(|o| o as u64),
        project_id: query.project_id,
    };

    let result = state.task_service.list(params).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            error = %e,
            "Failed to list tasks"
        );
        ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
            .with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(TaskListResponse {
        tasks: result.tasks,
        total: result.total,
        next_cursor: result.next_cursor,
        has_more: result.has_more,
    }))
}

pub async fn get_task(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(task_id): Path<Uuid>,
) -> ApiResult<Json<Task>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let task = state.task_service.get(&task_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            task_id = %task_id,
            error = %e,
            "Failed to get task"
        );

        let error_response = match e {
            ServerError::TaskNotFound(id) => ErrorResponse::new(
                ErrorCode::TaskNotFound,
                format!("Task with ID {} not found", id),
            )
            .with_metadata("task_id", serde_json::json!(id)),
            other => ErrorResponse::from(other),
        };

        error_response.with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(task))
}

pub async fn cancel_task(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(task_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let task = state.task_service.get(&task_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            task_id = %task_id,
            error = %e,
            "Failed to get task for cancellation"
        );
        ErrorResponse::from(e).with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    if let Some(worker_id) = task.claimed_by {
        if let Err(e) = state
            .coordinator
            .queue_cancellation(worker_id, task_id, "Cancelled by user".to_string())
            .await
        {
            tracing::warn!(
                correlation_id = ?correlation_id,
                task_id = %task_id,
                worker_id = %worker_id,
                error = %e,
                "Failed to queue cancellation to worker"
            );
        }
    }

    state
        .task_service
        .update_status(&task_id, TaskStatus::Cancelled)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                task_id = %task_id,
                error = %e,
                "Failed to cancel task"
            );
            ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        task_id = %task_id,
        "Task cancelled successfully"
    );

    Ok(StatusCode::NO_CONTENT)
}

pub async fn submit_feedback(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<SubmitFeedbackRequest>,
) -> ApiResult<StatusCode> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    tracing::info!(
        correlation_id = ?correlation_id,
        task_id = %task_id,
        feedback_type = ?payload.feedback_type,
        "Feedback submitted for task"
    );

    let feedback = HumanFeedback {
        provided_at: chrono::Utc::now(),
        feedback_type: payload.feedback_type,
        message: payload.message,
    };

    let task = state.task_service.get(&task_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            task_id = %task_id,
            error = %e,
            "Failed to get task for feedback"
        );
        ErrorResponse::from(e).with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    match task.claimed_by {
        Some(worker_id) => {
            state
                .coordinator
                .queue_feedback(worker_id, task_id, feedback.clone())
                .await
                .map_err(|e| {
                    tracing::error!(
                        correlation_id = ?correlation_id,
                        task_id = %task_id,
                        worker_id = %worker_id,
                        error = %e,
                        "Failed to submit feedback for task"
                    );
                    ErrorResponse::new(ErrorCode::InternalError, "Failed to queue feedback")
                        .with_details(e.to_string())
                        .with_correlation_id(correlation_id.unwrap_or_default())
                })?;

            if matches!(
                feedback.feedback_type,
                parallel_protocol::FeedbackType::RequestChanges
            ) {
                if let Err(e) = state
                    .task_service
                    .update_status(&task_id, TaskStatus::PendingRework)
                    .await
                {
                    tracing::error!(
                        correlation_id = ?correlation_id,
                        task_id = %task_id,
                        error = %e,
                        "Failed to update task status to PendingRework"
                    );
                }
            }

            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            tracing::warn!(
                correlation_id = ?correlation_id,
                task_id = %task_id,
                "Feedback rejected for unclaimed task"
            );
            Err(ErrorResponse::new(
                ErrorCode::FeedbackRejected,
                "Cannot submit feedback for unclaimed task",
            )
            .with_metadata("task_id", serde_json::json!(task_id))
            .with_correlation_id(correlation_id.unwrap_or_default()))
        }
    }
}

pub async fn get_review_data(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(task_id): Path<Uuid>,
) -> ApiResult<Json<Option<ReviewData>>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let review_data = state.task_service.get_review_data(&task_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            task_id = %task_id,
            error = %e,
            "Failed to get review data for task"
        );
        ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
            .with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(review_data))
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskStatusRequest>,
) -> ApiResult<StatusCode> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    state
        .task_service
        .complete_iteration(&task_id, payload.status)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                task_id = %task_id,
                status = ?payload.status,
                error = %e,
                "Failed to update task status"
            );
            ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        task_id = %task_id,
        status = ?payload.status,
        "Task status updated"
    );

    Ok(StatusCode::NO_CONTENT)
}

pub async fn retry_task(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<RetryTaskRequest>,
) -> ApiResult<Json<RetryTaskResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    tracing::info!(
        correlation_id = ?correlation_id,
        task_id = %task_id,
        clear_review_data = ?payload.clear_review_data,
        "Retrying task"
    );

    let clear_review_data = payload.clear_review_data.unwrap_or(false);

    let task = state
        .task_service
        .retry_task(&task_id, clear_review_data)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                task_id = %task_id,
                error = %e,
                "Failed to retry task"
            );

            let error_response = match e {
                ServerError::InvalidStatus(msg) => ErrorResponse::new(
                    ErrorCode::TaskNotRetryable,
                    msg,
                )
                .with_metadata("task_id", serde_json::json!(task_id)),
                ServerError::TaskNotFound(id) => ErrorResponse::new(
                    ErrorCode::TaskNotFound,
                    format!("Task with ID {} not found", id),
                )
                .with_metadata("task_id", serde_json::json!(id)),
                other => ErrorResponse::from(other),
            };

            error_response.with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        task_id = %task_id,
        status = ?task.status,
        "Task retried successfully"
    );

    Ok(Json(RetryTaskResponse {
        task_id: task.id,
        status: task.status,
    }))
}
