use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissions {
    pub can_send_messages: bool,
    pub can_control_terminal: bool,
    pub can_abort_task: bool,
    pub can_accept_work: bool,
}

impl Default for SessionPermissions {
    fn default() -> Self {
        Self {
            can_send_messages: true,
            can_control_terminal: true,
            can_abort_task: true,
            can_accept_work: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanSession {
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub attached_at: DateTime<Utc>,
    pub permissions: SessionPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub task_id: Uuid,
    pub permissions: Option<SessionPermissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
}
