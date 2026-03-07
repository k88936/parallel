use axum::{
    Json,
    extract::State,
    Extension,
};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use parallel_protocol::*;

use crate::api_error::{ApiResult, ErrorResponse};
use crate::error_codes::ErrorCode;
use crate::errors::ServerError;
use crate::state::AppState;

pub async fn register_worker(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<RegisterWorkerRequest>,
) -> ApiResult<Json<WorkerInfo>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    tracing::info!(
        correlation_id = ?correlation_id,
        worker_name = %payload.name,
        max_concurrent = payload.max_concurrent,
        "Registering worker"
    );

    let worker_info = state
        .worker_service
        .register(payload.name, payload.capabilities, payload.max_concurrent)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                error = %e,
                "Failed to register worker"
            );
            ErrorResponse::new(ErrorCode::WorkerRegistrationFailed, "Failed to register worker")
                .with_details(e.to_string())
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    tracing::info!(
        correlation_id = ?correlation_id,
        worker_id = %worker_info.id,
        "Worker registered successfully"
    );

    Ok(Json(worker_info))
}

pub async fn poll_instructions(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<PollRequest>,
) -> ApiResult<Json<PollResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let mut instructions = state
        .coordinator
        .get_pending_instructions(&payload.worker_id)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                worker_id = %payload.worker_id,
                error = %e,
                "Failed to poll instructions"
            );
            ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    if !instructions.is_empty() {
        return Ok(Json(PollResponse { instructions }));
    }

    let has_slot = match state.worker_service.has_available_slot(&payload.worker_id).await {
        Ok(has_slot) => has_slot,
        Err(e) => {
            tracing::error!(
                correlation_id = ?correlation_id,
                worker_id = %payload.worker_id,
                error = %e,
                "Failed to check available slot"
            );
            return Ok(Json(PollResponse { instructions }));
        }
    };

    if !has_slot {
        return Ok(Json(PollResponse { instructions }));
    }

    let Some(task) = state.task_service.get_next_queued().await.ok().flatten() else {
        return Ok(Json(PollResponse { instructions }));
    };

    if let Err(e) = state.worker_service.add_task(&payload.worker_id, task.id).await {
        tracing::error!(
            correlation_id = ?correlation_id,
            worker_id = %payload.worker_id,
            task_id = %task.id,
            error = %e,
            "Failed to add task to worker"
        );
        return Ok(Json(PollResponse { instructions }));
    }

    if let Err(e) = state
        .task_service
        .set_claimed_by(&task.id, Some(payload.worker_id))
        .await
    {
        tracing::error!(
            correlation_id = ?correlation_id,
            worker_id = %payload.worker_id,
            task_id = %task.id,
            error = %e,
            "Failed to set claimed_by"
        );
        return Ok(Json(PollResponse { instructions }));
    }

    tracing::info!(
        correlation_id = ?correlation_id,
        worker_id = %payload.worker_id,
        task_id = %task.id,
        "Task assigned to worker"
    );

    instructions.push(WorkerInstruction::AssignTask { task });

    Ok(Json(PollResponse { instructions }))
}

pub async fn push_events(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<PushEventsRequest>,
) -> ApiResult<Json<PushEventsResponse>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    tracing::debug!(
        correlation_id = ?correlation_id,
        worker_id = %payload.worker_id,
        event_count = payload.events.len(),
        "Processing worker events"
    );

    state
        .event_processor
        .process_events(&payload.worker_id, payload.events)
        .await
        .map_err(|e| {
            tracing::error!(
                correlation_id = ?correlation_id,
                worker_id = %payload.worker_id,
                error = %e,
                "Failed to process events"
            );
            ErrorResponse::new(ErrorCode::InternalError, "Failed to process events")
                .with_details(e.to_string())
                .with_correlation_id(correlation_id.unwrap_or_default())
        })?;

    Ok(Json(PushEventsResponse { acknowledged: true }))
}

pub async fn list_workers(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> ApiResult<Json<Vec<WorkerInfo>>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let workers = state.worker_service.list().await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            error = %e,
            "Failed to list workers"
        );
        ErrorResponse::from(ServerError::DatabaseError(e.to_string()))
            .with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(workers))
}
