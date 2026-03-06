use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Created,
    Queued,
    Claimed,
    InProgress,
    AwaitingReview,
    PendingRework,
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
            TaskStatus::PendingRework => "pending_rework",
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
            "pending_rework" => Some(TaskStatus::PendingRework),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub repo_url: String,
    pub description: String,
    pub base_branch: String,
    pub target_branch: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub claimed_by: Option<Uuid>,
    pub ssh_key: String,
    pub max_execution_time: i64,
}
