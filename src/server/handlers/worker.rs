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
        current_task: Set(None),
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
        current_task: None,
        capabilities: payload.capabilities,
        max_concurrent: payload.max_concurrent,
    }))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, StatusCode> {
    let now = Utc::now();

    let worker = workers::Entity::find_by_id(payload.worker_id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut worker: workers::ActiveModel = worker.into();
    worker.last_heartbeat = Set(now);
    worker.current_task = Set(payload.current_task);
    worker.status = Set(if payload.current_task.is_some() {
        WorkerStatus::Busy.as_str().to_string()
    } else {
        WorkerStatus::Idle.as_str().to_string()
    });

    worker.update(&state.db).await.map_err(|e| {
        tracing::error!("Failed to update heartbeat: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(HeartbeatResponse { acknowledged: true }))
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

            WorkerInfo {
                id: w.id,
                name: w.name,
                status: WorkerStatus::from_str(&w.status).unwrap_or(WorkerStatus::Offline),
                last_heartbeat: w.last_heartbeat,
                current_task: w.current_task,
                capabilities,
                max_concurrent: w.max_concurrent as usize,
            }
        })
        .collect();

    Ok(Json(worker_infos))
}