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
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::protocol::{HumanMessage, ServerMessage};
use crate::server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct HumanWsQuery {
    pub task_id: Uuid,
}

pub async fn human_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<HumanWsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_human_websocket(socket, state, query.task_id))
}

pub async fn handle_human_websocket(socket: WebSocket, state: AppState, task_id: Uuid) {
    info!("Human WebSocket connection established for task {}", task_id);

    let (mut ws_tx, mut ws_rx) = socket.split();

    let session_id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    state.ws_state.add_human(task_id, session_id, tx.clone()).await;

    if let Some(worker_tx) = state.ws_state.get_worker_for_task(task_id).await {
        let msg = ServerMessage::HumanAttached {
            task_id,
            session_id,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = worker_tx.send(Message::Text(json));
            info!("Notified worker about human attachment to task {}", task_id);
        }
    } else {
        warn!("No worker found for task {} when human attached", task_id);
    }

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
                    match serde_json::from_str::<HumanMessage>(&text) {
                        Ok(human_msg) => {
                            handle_human_message(&state, task_id, session_id, human_msg).await;
                        }
                        Err(e) => {
                            error!("Failed to parse human message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Human WebSocket error for task {}: {}", task_id, e);
                break;
            }
        }
    }

    send_task.abort();
    state.ws_state.remove_human(task_id, session_id).await;
    
    if let Some(worker_tx) = state.ws_state.get_worker_for_task(task_id).await {
        let msg = ServerMessage::HumanDetached {
            task_id,
            session_id,
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = worker_tx.send(Message::Text(json));
            info!("Notified worker about human detachment from task {}", task_id);
        }
    }
    
    info!("Human WebSocket connection closed for task {}", task_id);
}

async fn handle_human_message(
    state: &AppState,
    task_id: Uuid,
    session_id: Uuid,
    msg: HumanMessage,
) {
    match msg {
        HumanMessage::SendMessage { task_id: tid, message } => {
            info!("Human sent message to task {}: {}", tid, message);
            
            if let Some(worker_tx) = state.ws_state.get_worker_for_task(tid).await {
                let server_msg = ServerMessage::HumanMessage {
                    task_id: tid,
                    session_id,
                    message,
                };
                if let Ok(json) = serde_json::to_string(&server_msg) {
                    if let Err(e) = worker_tx.send(Message::Text(json)) {
                        error!("Failed to send human message to worker: {}", e);
                    }
                }
            } else {
                warn!("No worker found for task {} to send human message", tid);
            }
        }
        HumanMessage::TerminalInput { task_id: tid, terminal_id, input } => {
            info!("Human sent terminal input to task {} terminal {}", tid, terminal_id);
            
            if let Some(worker_tx) = state.ws_state.get_worker_for_task(tid).await {
                let server_msg = ServerMessage::TerminalCommand {
                    task_id: tid,
                    terminal_id,
                    command: input,
                };
                if let Ok(json) = serde_json::to_string(&server_msg) {
                    if let Err(e) = worker_tx.send(Message::Text(json)) {
                        error!("Failed to send terminal command to worker: {}", e);
                    }
                }
            }
        }
        HumanMessage::AbortTask { task_id: tid } => {
            warn!("Human requested abort for task {}", tid);
            
            if let Some(worker_tx) = state.ws_state.get_worker_for_task(tid).await {
                let server_msg = ServerMessage::AbortTask {
                    task_id: tid,
                    reason: "Human requested abort".to_string(),
                };
                if let Ok(json) = serde_json::to_string(&server_msg) {
                    if let Err(e) = worker_tx.send(Message::Text(json)) {
                        error!("Failed to send abort command to worker: {}", e);
                    }
                }
            }
        }
        HumanMessage::AcceptWork { task_id: tid } => {
            info!("Human accepted work for task {}", tid);
            
            if let Some(worker_tx) = state.ws_state.get_worker_for_task(tid).await {
                let server_msg = ServerMessage::AcceptWork {
                    task_id: tid,
                };
                if let Ok(json) = serde_json::to_string(&server_msg) {
                    if let Err(e) = worker_tx.send(Message::Text(json)) {
                        error!("Failed to send accept work command to worker: {}", e);
                    }
                }
            }
        }
    }
}