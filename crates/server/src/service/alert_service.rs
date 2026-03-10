use parallel_common::{Alert, AlertPayload};
use tokio::sync::broadcast;

const ALERT_CHANNEL_CAPACITY: usize = 1024;

#[derive(Clone)]
pub struct AlertService {
    tx: broadcast::Sender<AlertPayload>,
}

impl AlertService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(ALERT_CHANNEL_CAPACITY);
        Self { tx }
    }

    pub fn emit(&self, alert: Alert) {
        let payload = AlertPayload {
            severity: alert.severity(),
            alert,
        };
        let _ = self.tx.send(payload);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AlertPayload> {
        self.tx.subscribe()
    }
}

impl Default for AlertService {
    fn default() -> Self {
        Self::new()
    }
}

pub trait AlertServiceTrait: Send + Sync {
    fn emit(&self, alert: Alert);
    fn subscribe(&self) -> broadcast::Receiver<AlertPayload>;
}

impl AlertServiceTrait for AlertService {
    fn emit(&self, alert: Alert) {
        self.emit(alert);
    }

    fn subscribe(&self) -> broadcast::Receiver<AlertPayload> {
        self.subscribe()
    }
}
