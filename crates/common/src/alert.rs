use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum Alert {
    WorkerOffline {
        #[ts(type = "string")]
        worker_id: Uuid,
        worker_name: String,
        #[ts(type = "string[]")]
        running_tasks: Vec<Uuid>,
        #[ts(as = "String")]
        timestamp: DateTime<Utc>,
    },
    WorkerOnline {
        #[ts(type = "string")]
        worker_id: Uuid,
        worker_name: String,
        #[ts(as = "String")]
        timestamp: DateTime<Utc>,
    },
    TaskTimeout {
        #[ts(type = "string")]
        task_id: Uuid,
        task_title: String,
        max_execution_time: i64,
        #[ts(as = "String")]
        timestamp: DateTime<Utc>,
    },
    TaskReviewRequested {
        #[ts(type = "string")]
        task_id: Uuid,
        task_title: String,
        #[ts(type = "string")]
        worker_id: Uuid,
        #[ts(as = "String")]
        timestamp: DateTime<Utc>,
    },
    TaskCompleted {
        #[ts(type = "string")]
        task_id: Uuid,
        task_title: String,
        #[ts(as = "String")]
        timestamp: DateTime<Utc>,
    },
    TaskFailed {
        #[ts(type = "string")]
        task_id: Uuid,
        task_title: String,
        error: String,
        #[ts(as = "String")]
        timestamp: DateTime<Utc>,
    },
    TaskCancelled {
        #[ts(type = "string")]
        task_id: Uuid,
        task_title: String,
        #[ts(as = "String")]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AlertPayload {
    pub alert: Alert,
    pub severity: AlertSeverity,
}
