use axum::extract::ws::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub type WebSocketSender = mpsc::UnboundedSender<Message>;

#[derive(Debug, Clone)]
pub struct WorkerConnection {
    pub worker_id: Uuid,
    pub sender: WebSocketSender,
    pub current_task: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct HumanConnection {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub sender: WebSocketSender,
}

#[derive(Debug, Default)]
pub struct WebSocketState {
    workers: RwLock<HashMap<Uuid, WorkerConnection>>,
    humans: RwLock<HashMap<Uuid, Vec<HumanConnection>>>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add_worker(&self, worker_id: Uuid, sender: WebSocketSender) {
        let conn = WorkerConnection {
            worker_id,
            sender,
            current_task: None,
        };
        
        let mut workers = self.workers.write().await;
        workers.insert(worker_id, conn);
        info!("Worker {} connected via WebSocket", worker_id);
    }

    pub async fn remove_worker(&self, worker_id: Uuid) {
        let mut workers = self.workers.write().await;
        if workers.remove(&worker_id).is_some() {
            info!("Worker {} disconnected from WebSocket", worker_id);
        }
    }

    pub async fn update_worker_task(&self, worker_id: Uuid, task_id: Option<Uuid>) {
        let mut workers = self.workers.write().await;
        if let Some(conn) = workers.get_mut(&worker_id) {
            conn.current_task = task_id;
            debug!("Worker {} current task updated to {:?}", worker_id, task_id);
        }
    }

    pub async fn get_worker_for_task(&self, task_id: Uuid) -> Option<WebSocketSender> {
        let workers = self.workers.read().await;
        workers
            .values()
            .find(|conn| conn.current_task == Some(task_id))
            .map(|conn| conn.sender.clone())
    }

    pub async fn add_human(&self, task_id: Uuid, session_id: Uuid, sender: WebSocketSender) {
        let conn = HumanConnection {
            session_id,
            task_id,
            sender,
        };
        
        let mut humans = self.humans.write().await;
        humans.entry(task_id).or_insert_with(Vec::new).push(conn);
        info!("Human session {} attached to task {}", session_id, task_id);
    }

    pub async fn remove_human(&self, task_id: Uuid, session_id: Uuid) {
        let mut humans = self.humans.write().await;
        if let Some(connections) = humans.get_mut(&task_id) {
            connections.retain(|conn| conn.session_id != session_id);
            if connections.is_empty() {
                humans.remove(&task_id);
            }
            info!("Human session {} detached from task {}", session_id, task_id);
        }
    }

    pub async fn broadcast_to_task(&self, task_id: Uuid, message: Message) {
        let humans = self.humans.read().await;
        if let Some(connections) = humans.get(&task_id) {
            debug!("Broadcasting message to {} human connections for task {}", connections.len(), task_id);
            for conn in connections {
                if let Err(e) = conn.sender.send(message.clone()) {
                    error!("Failed to send message to human session {}: {}", conn.session_id, e);
                }
            }
        } else {
            debug!("No human connections for task {}", task_id);
        }
    }

    pub async fn send_to_worker(&self, worker_id: Uuid, message: Message) -> Result<(), String> {
        let workers = self.workers.read().await;
        if let Some(conn) = workers.get(&worker_id) {
            conn.sender
                .send(message)
                .map_err(|e| format!("Failed to send message to worker {}: {}", worker_id, e))
        } else {
            Err(format!("Worker {} not connected", worker_id))
        }
    }

    pub async fn get_human_count(&self, task_id: Uuid) -> usize {
        let humans = self.humans.read().await;
        humans.get(&task_id).map(|c| c.len()).unwrap_or(0)
    }
}