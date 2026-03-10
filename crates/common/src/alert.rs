use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Alert {
    WorkerOffline {
        worker_id: Uuid,
        worker_name: String,
        running_tasks: Vec<Uuid>,
        timestamp: DateTime<Utc>,
    },
    WorkerOnline {
        worker_id: Uuid,
        worker_name: String,
        timestamp: DateTime<Utc>,
    },
    TaskTimeout {
        task_id: Uuid,
        task_title: String,
        max_execution_time: i64,
        timestamp: DateTime<Utc>,
    },
    TaskReviewRequested {
        task_id: Uuid,
        task_title: String,
        worker_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    TaskCompleted {
        task_id: Uuid,
        task_title: String,
        timestamp: DateTime<Utc>,
    },
    TaskFailed {
        task_id: Uuid,
        task_title: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    TaskCancelled {
        task_id: Uuid,
        task_title: String,
        timestamp: DateTime<Utc>,
    },
}

impl Alert {
    pub fn severity(&self) -> AlertSeverity {
        match self {
            Alert::WorkerOffline { .. } => AlertSeverity::Error,
            Alert::WorkerOnline { .. } => AlertSeverity::Info,
            Alert::TaskTimeout { .. } => AlertSeverity::Error,
            Alert::TaskReviewRequested { .. } => AlertSeverity::Warning,
            Alert::TaskCompleted { .. } => AlertSeverity::Info,
            Alert::TaskFailed { .. } => AlertSeverity::Error,
            Alert::TaskCancelled { .. } => AlertSeverity::Info,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Alert::WorkerOffline { timestamp, .. } => *timestamp,
            Alert::WorkerOnline { timestamp, .. } => *timestamp,
            Alert::TaskTimeout { timestamp, .. } => *timestamp,
            Alert::TaskReviewRequested { timestamp, .. } => *timestamp,
            Alert::TaskCompleted { timestamp, .. } => *timestamp,
            Alert::TaskFailed { timestamp, .. } => *timestamp,
            Alert::TaskCancelled { timestamp, .. } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPayload {
    pub alert: Alert,
    pub severity: AlertSeverity,
}
