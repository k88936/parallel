use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::protocol::{HumanNotification, ServerMessage, WorkerMessage};
use crate::server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WorkerWsQuery {
    pub worker_id: Uuid,
}

pub async fn worker_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WorkerWsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_worker_websocket(socket, state, query.worker_id))
}

pub async fn handle_worker_websocket(socket: WebSocket, state: AppState, worker_id: Uuid) {
    info!("Worker {} WebSocket connection established", worker_id);

    let (mut ws_tx, mut ws_rx) = socket.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    state.ws_state.add_worker(worker_id, tx.clone()).await;

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg_result) = ws_rx.next().await {
        match msg_result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<WorkerMessage>(&text) {
                        Ok(worker_msg) => {
                            handle_worker_message(&state, worker_id, worker_msg, &tx).await;
                        }
                        Err(e) => {
                            error!("Failed to parse worker message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Worker {} WebSocket error: {}", worker_id, e);
                break;
            }
        }
    }

    send_task.abort();
    state.ws_state.remove_worker(worker_id).await;
    info!("Worker {} WebSocket connection closed", worker_id);
}

async fn handle_worker_message(
    state: &AppState,
    worker_id: Uuid,
    msg: WorkerMessage,
    tx: &mpsc::UnboundedSender<Message>,
) {
    match msg {
        WorkerMessage::TaskProgress { task_id, progress } => {
            debug!("Worker {} task {} progress: {:?}", worker_id, task_id, progress);
            state.ws_state.update_worker_task(worker_id, Some(task_id)).await;
            
            let notification = crate::protocol::HumanNotification::TaskProgress {
                task_id,
                update: progress,
            };
            if let Ok(json) = serde_json::to_string(&notification) {
                state.ws_state.broadcast_to_task(task_id, Message::Text(json)).await;
            }
        }
        WorkerMessage::AgentOutput { task_id, output } => {
            debug!("Worker {} task {} agent output: {} bytes", worker_id, task_id, output.len());
            
            let notification = crate::protocol::HumanNotification::AgentOutput {
                task_id,
                output,
            };
            if let Ok(json) = serde_json::to_string(&notification) {
                state.ws_state.broadcast_to_task(task_id, Message::Text(json)).await;
            }
        }
        WorkerMessage::TerminalOutput { task_id, terminal_id, output } => {
            debug!("Worker {} task {} terminal {} output: {} bytes", worker_id, task_id, terminal_id, output.len());
            
            let notification = crate::protocol::HumanNotification::TerminalOutput {
                task_id,
                terminal_id,
                output,
            };
            if let Ok(json) = serde_json::to_string(&notification) {
                state.ws_state.broadcast_to_task(task_id, Message::Text(json)).await;
            }
        }
        WorkerMessage::TaskCompleted { task_id, result } => {
            info!("Worker {} task {} completed with status {:?}", worker_id, task_id, result.status);
            
            let notification = crate::protocol::HumanNotification::TaskCompleted {
                task_id,
                branch: result.commits.first().cloned().unwrap_or_default(),
            };
            if let Ok(json) = serde_json::to_string(&notification) {
                state.ws_state.broadcast_to_task(task_id, Message::Text(json)).await;
            }
            
            state.ws_state.update_worker_task(worker_id, None).await;
        }
        WorkerMessage::TaskStatusUpdate { task_id, status } => {
            info!("Worker {} task {} status update: {:?}", worker_id, task_id, status);
            
            let notification = crate::protocol::HumanNotification::TaskStatusUpdate {
                task_id,
                status,
            };
            if let Ok(json) = serde_json::to_string(&notification) {
                state.ws_state.broadcast_to_task(task_id, Message::Text(json)).await;
            }
        }
        WorkerMessage::Heartbeat { worker_id: wid } => {
            debug!("Worker {} heartbeat", wid);
        }
    }
}