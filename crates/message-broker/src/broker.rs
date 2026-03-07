use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
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
pub struct MessageBroker {
    channels: Arc<DashMap<Uuid, WorkerChannel>>,
}

impl MessageBroker {
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
}

impl Default for MessageBroker {
    fn default() -> Self {
        Self::new()
    }
}
