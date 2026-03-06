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
