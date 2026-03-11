use axum::{
    Extension, Json,
    extract::{State, ws::{WebSocketUpgrade}, Path},
    http::{HeaderMap, header::AUTHORIZATION},
    response::Response,
};
use tower_http::request_id::RequestId;
use uuid::Uuid;

use parallel_common::*;

use crate::api_error::{ApiResult, ErrorResponse};
use crate::error_codes::ErrorCode;
use crate::errors::ServerError;
use crate::state::AppState;

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get(AUTHORIZATION)?.to_str().ok()?;
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        None
    }
}

pub async fn worker_websocket(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let token = extract_bearer_token(&headers).unwrap_or_default();

    let worker_info = match state.worker_service.get_by_token(&token).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!(
                correlation_id = ?correlation_id,
                error = %e,
                "Invalid token for WebSocket connection"
            );
            return Response::builder()
                .status(401)
                .body(axum::body::Body::from("Unauthorized"))
                .unwrap();
        }
    };

    let worker_id = worker_info.id;
    let worker_name = worker_info.name.clone();

    tracing::info!(
        correlation_id = ?correlation_id,
        worker_id = %worker_id,
        worker_name = %worker_name,
        "Worker WebSocket connection initiated"
    );

    let event_processor = state.event_processor.clone();
    let broker = state.message_broker.clone();

    ws.on_upgrade(move |socket| {
        let mut event_rx = broker.handle_connection(socket, worker_id);

        async move {
            while let Some(text) = event_rx.recv().await {
                match serde_json::from_str::<WorkerEvent>(&text) {
                    Ok(event) => {
                        if let Err(e) = event_processor.process_events(&worker_id, vec![event]).await {
                            tracing::error!(
                                correlation_id = ?correlation_id,
                                worker_id = %worker_id,
                                error = %e,
                                "Failed to process WebSocket event"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            correlation_id = ?correlation_id,
                            worker_id = %worker_id,
                            error = %e,
                            text = %text,
                            "Failed to parse WebSocket event"
                        );
                    }
                }
            }
        }
    })
}

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
            ErrorResponse::new(
                ErrorCode::WorkerRegistrationFailed,
                "Failed to register worker",
            )
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

pub async fn list_workers(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
) -> ApiResult<Json<Vec<WorkerSummary>>> {
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

    let summaries: Vec<WorkerSummary> = workers
        .into_iter()
        .map(|w| WorkerSummary {
            id: w.id,
            name: w.name,
            status: w.status,
            last_heartbeat: w.last_heartbeat,
            current_task_count: w.current_tasks.len(),
        })
        .collect();

    Ok(Json(summaries))
}

pub async fn get_worker_info(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(worker_id): Path<Uuid>,
) -> ApiResult<Json<WorkerInfo>> {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let worker = state.worker_service.get(&worker_id).await.map_err(|e| {
        tracing::error!(
            correlation_id = ?correlation_id,
            worker_id = %worker_id,
            error = %e,
            "Failed to get worker info"
        );
        ErrorResponse::new(ErrorCode::WorkerNotFound, "Worker not found")
            .with_correlation_id(correlation_id.unwrap_or_default())
    })?;

    Ok(Json(worker))
}

pub async fn get_worker_resources(
    State(state): State<AppState>,
    Extension(request_id): Extension<RequestId>,
    Path(worker_id): Path<Uuid>,
) -> ApiResult<Json<ResourceMonitor>> {
    let _correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let resources = state
        .worker_resources
        .get(&worker_id)
        .map(|r| r.clone())
        .ok_or_else(|| {
            ErrorResponse::new(ErrorCode::WorkerNotFound, "Worker resources not found")
        })?;

    Ok(Json(resources))
}
