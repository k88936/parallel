use chrono::{DateTime, Utc};
use uuid::Uuid;

use parallel_domain::{TaskPriority, TaskStatus};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
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
