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
    Iterating,
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
            TaskStatus::Iterating => "iterating",
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
            "iterating" => Some(TaskStatus::Iterating),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IterationStatus {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    Approve,
    RequestChanges,
    Abort,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub timestamp: DateTime<Utc>,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFeedback {
    pub provided_at: DateTime<Utc>,
    pub feedback_type: FeedbackType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub repo_url: String,
    pub description: String,
    pub base_branch: Option<String>,
    pub target_branch: Option<String>,
    pub priority: Option<TaskPriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<TaskStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitFeedbackRequest {
    pub feedback_type: FeedbackType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimTaskRequest {
    pub worker_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimTaskResponse {
    pub task: Option<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerInstruction {
    AssignTask { task: Task },
    CancelTask { task_id: Uuid, reason: String },
    UpdateTask { task_id: Uuid, instruction: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerEvent {
    Heartbeat { running_tasks: Vec<Uuid> },
    TaskStarted { task_id: Uuid },
    TaskProgress { task_id: Uuid, message: String },
    TaskCompleted { task_id: Uuid },
    TaskFailed { task_id: Uuid, error: String },
    TaskCancelled { task_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollRequest {
    pub worker_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResponse {
    pub instructions: Vec<WorkerInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventsRequest {
    pub worker_id: Uuid,
    pub events: Vec<WorkerEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventsResponse {
    pub acknowledged: bool,
}
