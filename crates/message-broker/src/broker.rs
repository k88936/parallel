use std::sync::Arc;
use dashmap::DashMap;
use parallel_protocol::{WorkerEvent, WorkerInstruction};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 256;

#[derive(Clone)]
pub struct WorkerChannel {
    instruction_tx: broadcast::Sender<Arc<WorkerInstruction>>,
    event_tx: mpsc::Sender<WorkerEvent>,
}

impl WorkerChannel {
    pub fn new() -> Self {
        let (instruction_tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        let (event_tx, _) = mpsc::channel(CHANNEL_CAPACITY);
        Self {
            instruction_tx,
            event_tx,
        }
    }

    pub fn send_instruction(&self, instruction: WorkerInstruction) {
        let _ = self.instruction_tx.send(Arc::new(instruction));
    }

    pub fn subscribe_instructions(&self) -> broadcast::Receiver<Arc<WorkerInstruction>> {
        self.instruction_tx.subscribe()
    }

    pub async fn send_event(&self, event: WorkerEvent) -> Result<(), mpsc::error::SendError<WorkerEvent>> {
        self.event_tx.send(event).await
    }

    pub fn event_receiver(&self) -> mpsc::Receiver<WorkerEvent> {
        let (_tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
        rx
    }

    pub fn instruction_subscriber_count(&self) -> usize {
        self.instruction_tx.receiver_count()
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
    events_tx: mpsc::Sender<(Uuid, WorkerEvent)>,
    events_rx: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<(Uuid, WorkerEvent)>>>>,
}

impl MessageBroker {
    pub fn new() -> Self {
        let (events_tx, events_rx) = mpsc::channel(CHANNEL_CAPACITY);
        Self {
            channels: Arc::new(DashMap::new()),
            events_tx,
            events_rx: Arc::new(tokio::sync::Mutex::new(Some(events_rx))),
        }
    }

    pub fn register_worker(&self, worker_id: Uuid) {
        let channel = WorkerChannel::new();
        self.channels.insert(worker_id, channel);
    }

    pub fn unregister_worker(&self, worker_id: &Uuid) {
        self.channels.remove(worker_id);
    }

    pub fn send_instruction(&self, worker_id: &Uuid, instruction: WorkerInstruction) -> bool {
        if let Some(channel) = self.channels.get(worker_id) {
            channel.send_instruction(instruction);
            true
        } else {
            false
        }
    }

    pub fn subscribe_instructions(&self, worker_id: &Uuid) -> Option<broadcast::Receiver<Arc<WorkerInstruction>>> {
        self.channels.get(worker_id).map(|c| c.subscribe_instructions())
    }

    pub async fn send_event(&self, worker_id: Uuid, event: WorkerEvent) -> Result<(), mpsc::error::SendError<(Uuid, WorkerEvent)>> {
        self.events_tx.send((worker_id, event)).await
    }

    pub async fn take_event_receiver(&self) -> Option<mpsc::Receiver<(Uuid, WorkerEvent)>> {
        self.events_rx.lock().await.take()
    }

    pub fn is_connected(&self, worker_id: &Uuid) -> bool {
        self.channels.get(worker_id).is_some()
    }

    pub fn connected_workers(&self) -> Vec<Uuid> {
        self.channels.iter().filter_map(|e| {
            if e.instruction_subscriber_count() > 0 {
                Some(*e.key())
            } else {
                None
            }
        }).collect()
    }
}

impl Default for MessageBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bidirectional() {
        let broker = MessageBroker::new();
        let worker_id = Uuid::new_v4();
        
        broker.register_worker(worker_id);
        
        let mut rx = broker.subscribe_instructions(&worker_id).unwrap();
        
        broker.send_instruction(&worker_id, WorkerInstruction::CancelTask {
            task_id: Uuid::new_v4(),
            reason: "test".to_string(),
        });
        
        let received = rx.recv().await.unwrap();
        match (*received).clone() {
            WorkerInstruction::CancelTask { reason, .. } => {
                assert_eq!(reason, "test");
            }
            _ => panic!("Wrong type"),
        }
    }
}
