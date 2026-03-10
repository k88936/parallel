use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 256;

#[derive(Clone)]
pub struct WorkerChannel {
    tx: broadcast::Sender<Arc<String>>,
}

impl WorkerChannel {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { tx }
    }

    pub fn send(&self, json: String) {
        let _ = self.tx.send(Arc::new(json));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<String>> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for WorkerChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct MessageBrokerServer {
    channels: Arc<DashMap<Uuid, WorkerChannel>>,
}

impl MessageBrokerServer {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, id: Uuid) {
        self.channels.insert(id, WorkerChannel::new());
    }

    pub fn unregister(&self, id: &Uuid) {
        self.channels.remove(id);
    }

    pub fn send(&self, id: &Uuid, json: String) -> bool {
        if let Some(channel) = self.channels.get(id) {
            channel.send(json);
            true
        } else {
            false
        }
    }

    pub fn subscribe(&self, id: &Uuid) -> Option<broadcast::Receiver<Arc<String>>> {
        self.channels.get(id).map(|c| c.subscribe())
    }

    pub fn is_connected(&self, id: &Uuid) -> bool {
        self.channels.get(id).is_some()
    }

    pub fn connected_ids(&self) -> Vec<Uuid> {
        self.channels
            .iter()
            .filter_map(|e| {
                if e.subscriber_count() > 0 {
                    Some(*e.key())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn handle_connection(
        self: Arc<Self>,
        mut socket: WebSocket,
        worker_id: Uuid,
    ) -> mpsc::Receiver<String> {
        let (event_tx, event_rx) = mpsc::channel(CHANNEL_CAPACITY);

        self.register(worker_id);

        let mut instruction_rx = match self.subscribe(&worker_id) {
            Some(rx) => rx,
            None => {
                tracing::error!(worker_id = %worker_id, "Failed to subscribe to message broker");
                return event_rx;
            }
        };

        let broker = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    instruction = instruction_rx.recv() => {
                        match instruction {
                            Ok(json) => {
                                if socket.send(Message::Text(Utf8Bytes::from((*json).clone()))).await.is_err() {
                                    tracing::warn!(worker_id = %worker_id, "Failed to send instruction via WebSocket");
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                tracing::info!(worker_id = %worker_id, "Instruction channel closed");
                                break;
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                tracing::warn!(worker_id = %worker_id, lagged = n, "Worker WebSocket lagged behind");
                            }
                        }
                    }

                    msg = socket.recv() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                let _ = event_tx.send(text.to_string()).await;
                            }
                            Some(Ok(Message::Ping(data))) => {
                                let _ = socket.send(Message::Pong(data)).await;
                            }
                            Some(Ok(Message::Close(_))) => {
                                tracing::info!(worker_id = %worker_id, "Worker WebSocket closed by client");
                                break;
                            }
                            Some(Err(e)) => {
                                tracing::error!(worker_id = %worker_id, error = %e, "WebSocket error");
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                }
            }

            broker.unregister(&worker_id);
            tracing::info!(worker_id = %worker_id, "Worker WebSocket disconnected");
        });

        event_rx
    }
}

impl Default for MessageBrokerServer {
    fn default() -> Self {
        Self::new()
    }
}
