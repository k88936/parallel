use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResourceMonitor {
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub disk_usage_percent: f32,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkerInfo {
    #[ts(type = "string")]
    pub id: Uuid,
    pub token: String,
    pub name: String,
    pub status: WorkerStatus,
    #[ts(as = "String")]
    pub last_heartbeat: DateTime<Utc>,
    #[ts(type = "string[]")]
    pub current_tasks: Vec<Uuid>,
    pub capabilities: WorkerCapabilities,
    pub max_concurrent: usize,
}
