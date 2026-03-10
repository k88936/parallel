use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Idle,
    Busy,
    Offline,
    Dead,
}

impl WorkerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerStatus::Idle => "idle",
            WorkerStatus::Busy => "busy",
            WorkerStatus::Offline => "offline",
            WorkerStatus::Dead => "dead",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(WorkerStatus::Idle),
            "busy" => Some(WorkerStatus::Busy),
            "offline" => Some(WorkerStatus::Offline),
            "dead" => Some(WorkerStatus::Dead),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub has_git: bool,
    pub available_agents: Vec<String>,
    pub supported_languages: Vec<String>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl Default for WorkerCapabilities {
    fn default() -> Self {
        Self {
            has_git: true,
            available_agents: vec![],
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
            ],
            labels: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub id: Uuid,
    pub token: String,
    pub name: String,
    pub status: WorkerStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub current_tasks: Vec<Uuid>,
    pub capabilities: WorkerCapabilities,
    pub max_concurrent: usize,
}
