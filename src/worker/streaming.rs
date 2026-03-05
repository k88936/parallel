use crate::protocol::{ExecutionStage, ServerMessage, TaskProgressUpdate, WorkerMessage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[async_trait]
pub trait ProgressReporter: Send + Sync {
    async fn report_progress(&self, task_id: Uuid, update: TaskProgressUpdate);
    async fn report_agent_output(&self, task_id: Uuid, output: String);
    async fn report_terminal_output(&self, task_id: Uuid, terminal_id: Uuid, output: String);
    async fn report_task_status(&self, task_id: Uuid, status: crate::protocol::TaskStatus);
}

pub struct WebSocketClient {
    worker_id: Uuid,
    ws_url: String,
    sender: mpsc::UnboundedSender<WorkerMessage>,
    server_messages: Arc<RwLock<Vec<ServerMessage>>>,
    connected: Arc<RwLock<bool>>,
}

impl WebSocketClient {
    pub async fn new(worker_id: Uuid, ws_url: String) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel::<WorkerMessage>();
        let server_messages = Arc::new(RwLock::new(Vec::new()));
        let connected = Arc::new(RwLock::new(false));

        let ws_url_with_id = format!("{}?worker_id={}", ws_url, worker_id);
        info!("Connecting to WebSocket: {}", ws_url_with_id);

        let (ws_stream, _) = connect_async(&ws_url_with_id)
            .await
            .context("Failed to connect to WebSocket")?;

        info!("WebSocket connected successfully");

        let (ws_tx, mut ws_rx) = ws_stream.split();
        let ws_tx = Arc::new(Mutex::new(ws_tx));

        let server_msgs = server_messages.clone();
        let conn_status = connected.clone();
        let ws_tx_for_recv = ws_tx.clone();
        
        tokio::spawn(async move {
            while let Some(msg_result) = ws_rx.next().await {
                match msg_result {
                    Ok(WsMessage::Text(text)) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(server_msg) => {
                                debug!("Received server message: {:?}", server_msg);
                                let mut msgs = server_msgs.write().await;
                                msgs.push(server_msg);
                            }
                            Err(e) => {
                                error!("Failed to parse server message: {}", e);
                            }
                        }
                    }
                    Ok(WsMessage::Ping(data)) => {
                        let mut tx = ws_tx_for_recv.lock().await;
                        let _ = tx.send(WsMessage::Pong(data)).await;
                    }
                    Ok(WsMessage::Close(_)) => {
                        warn!("WebSocket closed by server");
                        *conn_status.write().await = false;
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket receive error: {}", e);
                        *conn_status.write().await = false;
                        break;
                    }
                    _ => {}
                }
            }
        });

        let conn_status = connected.clone();
        let mut rx = rx;
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let mut tx = ws_tx.lock().await;
                    if tx.send(WsMessage::Text(json)).await.is_err() {
                        error!("Failed to send WebSocket message");
                        *conn_status.write().await = false;
                        break;
                    }
                }
            }
        });

        *connected.write().await = true;

        Ok(Self {
            worker_id,
            ws_url,
            sender: tx,
            server_messages,
            connected,
        })
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    pub async fn send_message(&self, msg: WorkerMessage) -> Result<()> {
        self.sender
            .send(msg)
            .context("Failed to send message to WebSocket")
    }

    pub async fn get_server_messages(&self) -> Vec<ServerMessage> {
        let mut msgs = self.server_messages.write().await;
        let result = msgs.clone();
        msgs.clear();
        result
    }

    pub async fn send_heartbeat(&self) -> Result<()> {
        let msg = WorkerMessage::Heartbeat {
            worker_id: self.worker_id,
        };
        self.send_message(msg).await
    }
}

#[async_trait]
impl ProgressReporter for WebSocketClient {
    async fn report_progress(&self, task_id: Uuid, update: TaskProgressUpdate) {
        let msg = WorkerMessage::TaskProgress {
            task_id,
            progress: update,
        };
        if let Err(e) = self.send_message(msg).await {
            error!("Failed to report progress: {}", e);
        }
    }

    async fn report_agent_output(&self, task_id: Uuid, output: String) {
        let msg = WorkerMessage::AgentOutput { task_id, output };
        if let Err(e) = self.send_message(msg).await {
            error!("Failed to report agent output: {}", e);
        }
    }

    async fn report_terminal_output(&self, task_id: Uuid, terminal_id: Uuid, output: String) {
        let msg = WorkerMessage::TerminalOutput {
            task_id,
            terminal_id,
            output,
        };
        if let Err(e) = self.send_message(msg).await {
            error!("Failed to report terminal output: {}", e);
        }
    }

    async fn report_task_status(&self, task_id: Uuid, status: crate::protocol::TaskStatus) {
        let msg = WorkerMessage::TaskStatusUpdate { task_id, status };
        if let Err(e) = self.send_message(msg).await {
            error!("Failed to report task status: {}", e);
        }
    }
}

pub struct NoOpReporter;

#[async_trait]
impl ProgressReporter for NoOpReporter {
    async fn report_progress(&self, _task_id: Uuid, _update: TaskProgressUpdate) {}
    async fn report_agent_output(&self, _task_id: Uuid, _output: String) {}
    async fn report_terminal_output(&self, _task_id: Uuid, _terminal_id: Uuid, _output: String) {}
    async fn report_task_status(&self, _task_id: Uuid, _status: crate::protocol::TaskStatus) {}
}