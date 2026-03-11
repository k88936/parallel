use crate::instructions::FeedbackType;
use crate::instructions::{WorkerEvent, WorkerInstruction};
use crate::project::{Project, RepoConfig, SshKeyConfig};
use crate::TaskStatus;
use crate::{Task, TaskPriority, WorkerCapabilities};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateTaskRequest {
    pub title: String,
    pub repo_url: Option<String>,
    pub repo_ref: Option<String>,
    pub description: String,
    pub base_branch: Option<String>,
    pub target_branch: Option<String>,
    pub priority: Option<TaskPriority>,
    pub ssh_key: Option<String>,
    pub ssh_key_ref: Option<String>,
    pub max_execution_time: Option<i64>,
    #[ts(optional, type = "string")]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub required_labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateTaskResponse {
    #[ts(type = "string")]
    pub task_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct TaskSort {
    pub field: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct ListTasksQuery {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub repo_url: Option<String>,
    #[ts(optional, type = "string")]
    pub worker_id: Option<Uuid>,
    pub search: Option<String>,
    #[ts(optional, as = "Option<String>")]
    pub created_after: Option<DateTime<Utc>>,
    #[ts(optional, as = "Option<String>")]
    pub created_before: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    #[ts(optional, type = "string")]
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
    pub total: u64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SubmitFeedbackRequest {
    pub feedback_type: FeedbackType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateProjectRequest {
    pub name: String,
    pub repos: Vec<RepoConfig>,
    pub ssh_keys: Vec<SshKeyConfig>,
    #[ts(optional, type = "string")]
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateProjectResponse {
    #[ts(type = "string")]
    pub project_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct ListProjectsQuery {
    pub search: Option<String>,
    pub sort_direction: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total: u64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub repos: Option<Vec<RepoConfig>>,
    pub ssh_keys: Option<Vec<SshKeyConfig>>,
    #[ts(optional, type = "string")]
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RegisterWorkerRequest {
    pub name: String,
    pub capabilities: WorkerCapabilities,
    pub max_concurrent: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PollRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PollResponse {
    pub instructions: Vec<WorkerInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PushEventsRequest {
    pub token: String,
    pub events: Vec<WorkerEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PushEventsResponse {
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RetryTaskRequest {
    pub clear_review_data: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RetryTaskResponse {
    #[ts(type = "string")]
    pub task_id: Uuid,
    pub status: TaskStatus,
}
