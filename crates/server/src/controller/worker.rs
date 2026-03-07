use axum::{
    Json,
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade, Message},
        Query,
    },
    Extension,
    response::Response,
    http::{header::AUTHORIZATION, HeaderMap},
};
use serde::Deserialize;
use tokio::sync::broadcast;
use tower_http::request_id::RequestId;
use uuid::Uuid;

use parallel_common::*;

use crate::api_error::{ApiResult, ErrorResponse};
use crate::error_codes::ErrorCode;
use crate::errors::ServerError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

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
    Query(query): Query<WsQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let correlation_id = request_id
        .header_value()
        .to_str()
        .ok()
        .and_then(|s| Uuid::parse_str(s).ok());

    let token = extract_bearer_token(&headers)
        .or(query.token)
        .unwrap_or_default();

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

    state.message_broker.register(worker_id);

    ws.on_upgrade(move |socket| handle_websocket(socket, state, worker_id, correlation_id))
}

async fn handle_websocket(
    mut socket: WebSocket,
    state: AppState,
    worker_id: Uuid,
    correlation_id: Option<Uuid>,
) {
    let mut instruction_rx = match state.message_broker.subscribe(&worker_id) {
        Some(rx) => rx,
        None => {
            tracing::error!(
                correlation_id = ?correlation_id,
                worker_id = %worker_id,
                "Failed to subscribe to message broker"
            );
            return;
        }
    };

    tracing::info!(
        correlation_id = ?correlation_id,
        worker_id = %worker_id,
        "Worker WebSocket connected, streaming instructions"
    );

    loop {
        tokio::select! {
            instruction = instruction_rx.recv() => {
                match instruction {
                    Ok(json) => {
                        if socket.send(Message::Text((*json).clone())).await.is_err() {
                            tracing::warn!(
                                correlation_id = ?correlation_id,
                                worker_id = %worker_id,
                                "Failed to send instruction via WebSocket"
                            );
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!(
                            correlation_id = ?correlation_id,
                            worker_id = %worker_id,
                            "Instruction channel closed"
                        );
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(
                            correlation_id = ?correlation_id,
                            worker_id = %worker_id,
                            lagged = n,
                            "Worker WebSocket lagged behind"
                        );
                    }
                }
            }

            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WorkerEvent>(&text) {
                            Ok(event) => {
                                if let Err(e) = state.event_processor.process_events(&worker_id, vec![event]).await {
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
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!(
                            correlation_id = ?correlation_id,
                            worker_id = %worker_id,
                            "Worker WebSocket closed by client"
                        );
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!(
                            correlation_id = ?correlation_id,
                            worker_id = %worker_id,
                            error = %e,
                            "WebSocket error"
                        );
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
        }
    }

    state.message_broker.unregister(&worker_id);
    tracing::info!(
        correlation_id = ?correlation_id,
        worker_id = %worker_id,
        "Worker WebSocket disconnected"
    );
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
