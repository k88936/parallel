use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::workers;

#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = workers)]
pub struct Worker {
    pub id: String,
    pub token: String,
    pub name: String,
    pub status: String,
    pub last_heartbeat: NaiveDateTime,
    pub current_tasks_json: String,
    pub pending_instructions_json: String,
    pub capabilities_json: String,
    pub max_concurrent: i32,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = workers)]
pub struct NewWorker {
    pub id: String,
    pub token: String,
    pub name: String,
    pub status: String,
    pub last_heartbeat: NaiveDateTime,
    pub current_tasks_json: String,
    pub pending_instructions_json: String,
    pub capabilities_json: String,
    pub max_concurrent: i32,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = workers)]
pub struct WorkerChangeset {
    pub token: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub current_tasks_json: Option<String>,
    pub pending_instructions_json: Option<String>,
    pub capabilities_json: Option<String>,
    pub max_concurrent: Option<i32>,
}

impl Worker {
    pub fn get_uuid(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap_or_default()
    }
}
