use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Task, TaskPriority, TaskStatus, WorkerCapabilities};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
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
    pub worker_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResponse {
    pub instructions: Vec<super::WorkerInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventsRequest {
    pub worker_id: Uuid,
    pub events: Vec<super::WorkerEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEventsResponse {
    pub acknowledged: bool,
}
