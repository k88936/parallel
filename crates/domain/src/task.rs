use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Created,
    Queued,
    Claimed,
    InProgress,
    AwaitingReview,
    PendingResponse,
    Completed,
    Cancelled,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Created => "created",
            TaskStatus::Queued => "queued",
            TaskStatus::Claimed => "claimed",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::AwaitingReview => "awaiting_review",
            TaskStatus::PendingResponse => "pending_response",
            TaskStatus::Completed => "completed",
            TaskStatus::Cancelled => "cancelled",
            TaskStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "created" => Some(TaskStatus::Created),
            "queued" => Some(TaskStatus::Queued),
            "claimed" => Some(TaskStatus::Claimed),
            "in_progress" => Some(TaskStatus::InProgress),
            "awaiting_review" => Some(TaskStatus::AwaitingReview),
            "pending_response" => Some(TaskStatus::PendingResponse),
            "completed" => Some(TaskStatus::Completed),
            "cancelled" => Some(TaskStatus::Cancelled),
            "failed" => Some(TaskStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl TaskPriority {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(TaskPriority::Low),
            1 => Some(TaskPriority::Normal),
            2 => Some(TaskPriority::High),
            3 => Some(TaskPriority::Urgent),
            _ => None,
        }
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}
