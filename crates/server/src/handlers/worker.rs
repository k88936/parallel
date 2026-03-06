use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use parallel_protocol::*;

use crate::services::{Coordinator, EventProcessor, TaskService, WorkerService};
use crate::state::AppState;

pub async fn register_worker(
    State(state): State<AppState>,
    Json(payload): Json<RegisterWorkerRequest>,
) -> Result<Json<WorkerInfo>, StatusCode> {
    let worker_service = WorkerService::new(state.db.clone());

    match worker_service
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
    let coordinator = Coordinator::new(state.db.clone());
    let worker_service = WorkerService::new(state.db.clone());
    let task_service = TaskService::new(state.db.clone());

    match coordinator.get_pending_instructions(&payload.worker_id).await {
        Ok(mut instructions) => {
            if instructions.is_empty() {
                match worker_service.has_available_slot(&payload.worker_id).await {
                    Ok(true) => {
                        if let Ok(Some(task)) = task_service.get_next_queued().await {
                            match worker_service.add_task(&payload.worker_id, task.id).await {
                                Ok(()) => {
                                    if let Err(e) = task_service
                                        .set_claimed_by(&task.id, Some(payload.worker_id))
                                        .await
                                    {
                                        tracing::error!("Failed to set claimed_by: {}", e);
                                    } else {
                                        instructions.push(WorkerInstruction::AssignTask { task });
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to add task to worker: {}", e);
                                }
                            }
                        }
                    }
                    Ok(false) => {}
                    Err(e) => {
                        tracing::error!("Failed to check available slot: {}", e);
                    }
                }
            }

            Ok(Json(PollResponse { instructions }))
        }
        Err(e) => {
            tracing::error!("Failed to poll instructions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn push_events(
    State(state): State<AppState>,
    Json(payload): Json<PushEventsRequest>,
) -> Result<Json<PushEventsResponse>, StatusCode> {
    let event_processor = EventProcessor::new(state.db.clone());

    match event_processor
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
    let worker_service = WorkerService::new(state.db.clone());

    match worker_service.list().await {
        Ok(workers) => Ok(Json(workers)),
        Err(e) => {
            tracing::error!("Failed to list workers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
