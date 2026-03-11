use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ROOT_PROJECT_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");

pub fn is_root_project_id(id: &Uuid) -> bool {
    *id == ROOT_PROJECT_ID
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyConfig {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub repos: Vec<RepoConfig>,
    pub ssh_keys: Vec<SshKeyConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parent_id: Option<Uuid>,
}

impl Project {
    pub fn is_root(&self) -> bool {
        is_root_project_id(&self.id)
    }
}
