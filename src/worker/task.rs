use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub repo_url: String,
    pub description: String,
    pub branch_name: String,
    pub ssh_key_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub branch_name: String,
    pub error: Option<String>,
}

impl TaskResult {
    pub fn success(task_id: String, branch_name: String) -> Self {
        Self {
            task_id,
            success: true,
            branch_name,
            error: None,
        }
    }

    pub fn failure(task_id: String, branch_name: String, error: String) -> Self {
        Self {
            task_id,
            success: false,
            branch_name,
            error: Some(error),
        }
    }
}
