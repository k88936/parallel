use axum::{Json, extract::State, http::StatusCode};

use parallel_protocol::*;

use crate::state::AppState;

pub async fn register_worker(
    State(state): State<AppState>,
    Json(payload): Json<RegisterWorkerRequest>,
) -> Result<Json<WorkerInfo>, StatusCode> {
    match state
        .worker_service
        .register(payload.name, payload.capabilities, payload.max_concurrent)
        .await
    {
        Ok(worker_info) => Ok(Json(worker_info)),
        Err(e) => {
            tracing::error!("Failed to register worker: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn poll_instructions(
    State(state): State<AppState>,
    Json(payload): Json<PollRequest>,
) -> Result<Json<PollResponse>, StatusCode> {
    let mut instructions = match state
        .coordinator
        .get_pending_instructions(&payload.worker_id)
        .await
    {
        Ok(instructions) => instructions,
        Err(e) => {
            tracing::error!("Failed to poll instructions: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !instructions.is_empty() {
        return Ok(Json(PollResponse { instructions }));
    }

    let has_slot = match state
        .worker_service
        .has_available_slot(&payload.worker_id)
        .await
    {
        Ok(has_slot) => has_slot,
        Err(e) => {
            tracing::error!("Failed to check available slot: {}", e);
            return Ok(Json(PollResponse { instructions }));
        }
    };

    if !has_slot {
        return Ok(Json(PollResponse { instructions }));
    }

    let Some(task) = state.task_service.get_next_queued().await.ok().flatten() else {
        return Ok(Json(PollResponse { instructions }));
    };

    if let Err(e) = state
        .worker_service
        .add_task(&payload.worker_id, task.id)
        .await
    {
        tracing::error!("Failed to add task to worker: {}", e);
        return Ok(Json(PollResponse { instructions }));
    }

    if let Err(e) = state
        .task_service
        .set_claimed_by(&task.id, Some(payload.worker_id))
        .await
    {
        tracing::error!("Failed to set claimed_by: {}", e);
        return Ok(Json(PollResponse { instructions }));
    }

    instructions.push(WorkerInstruction::AssignTask { task });

    Ok(Json(PollResponse { instructions }))
}

pub async fn push_events(
    State(state): State<AppState>,
    Json(payload): Json<PushEventsRequest>,
) -> Result<Json<PushEventsResponse>, StatusCode> {
    match state
        .event_processor
        .process_events(&payload.worker_id, payload.events)
        .await
    {
        Ok(()) => Ok(Json(PushEventsResponse { acknowledged: true })),
        Err(e) => {
            tracing::error!("Failed to process events: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_workers(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkerInfo>>, StatusCode> {
    match state.worker_service.list().await {
        Ok(workers) => Ok(Json(workers)),
        Err(e) => {
            tracing::error!("Failed to list workers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
