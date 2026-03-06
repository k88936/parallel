use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use crate::protocol::*;
use crate::server::db::entity::workers;
use crate::server::state::AppState;

pub async fn register_worker(
    State(state): State<AppState>,
    Json(payload): Json<RegisterWorkerRequest>,
) -> Result<Json<WorkerInfo>, StatusCode> {
    let worker_id = Uuid::new_v4();
    let now = Utc::now();
    let capabilities_json = serde_json::to_string(&payload.capabilities)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let worker = workers::ActiveModel {
        id: Set(worker_id),
        name: Set(payload.name.clone()),
        status: Set(WorkerStatus::Idle.as_str().to_string()),
        last_heartbeat: Set(now),
        current_tasks_json: Set("[]".to_string()),
        pending_instructions_json: Set("[]".to_string()),
        capabilities_json: Set(capabilities_json.clone()),
        max_concurrent: Set(payload.max_concurrent as i32),
    };

    workers::Entity::insert(worker)
        .exec(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to register worker: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(WorkerInfo {
        id: worker_id,
        name: payload.name,
        status: WorkerStatus::Idle,
        last_heartbeat: now,
        current_tasks: vec![],
        capabilities: payload.capabilities,
        max_concurrent: payload.max_concurrent,
    }))
}

pub async fn poll_instructions(
    State(state): State<AppState>,
    Json(payload): Json<PollRequest>,
) -> Result<Json<PollResponse>, StatusCode> {
    match state.scheduler.poll_instructions(&payload.worker_id).await {
        Ok(instructions) => Ok(Json(PollResponse { instructions })),
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
    match state.scheduler.process_events(&payload.worker_id, payload.events).await {
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
    let workers = workers::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list workers: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let worker_infos: Vec<WorkerInfo> = workers
        .into_iter()
        .map(|w| {
            let capabilities: WorkerCapabilities = serde_json::from_str(&w.capabilities_json)
                .unwrap_or_default();
            let current_tasks: Vec<Uuid> = serde_json::from_str(&w.current_tasks_json)
                .unwrap_or_default();

            WorkerInfo {
                id: w.id,
                name: w.name,
                status: WorkerStatus::from_str(&w.status).unwrap_or(WorkerStatus::Offline),
                last_heartbeat: w.last_heartbeat,
                current_tasks,
                capabilities,
                max_concurrent: w.max_concurrent as usize,
            }
        })
        .collect();

    Ok(Json(worker_infos))
}
