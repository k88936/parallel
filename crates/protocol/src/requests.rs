use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Task, TaskPriority, TaskStatus, WorkerCapabilities};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub repo_url: String,
    pub description: String,
    pub base_branch: Option<String>,
    pub target_branch: Option<String>,
    pub priority: Option<TaskPriority>,
    pub ssh_key: String,
    pub max_execution_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskSort {
    pub field: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListTasksQuery {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub repo_url: Option<String>,
    pub worker_id: Option<Uuid>,
    pub search: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
    pub total: u64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitFeedbackRequest {
    pub feedback_type: super::FeedbackType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerRequest {
    pub name: String,
    pub capabilities: WorkerCapabilities,
    pub max_concurrent: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResponse {
    pub instructions: Vec<super::WorkerInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventsRequest {
    pub token: String,
    pub events: Vec<super::WorkerEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventsResponse {
    pub acknowledged: bool,
}
