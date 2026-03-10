use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::tasks;

#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub repo_url: String,
    pub description: String,
    pub base_branch: String,
    pub target_branch: String,
    pub status: String,
    pub priority: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub claimed_by: Option<String>,
    pub review_data_json: Option<String>,
    pub ssh_key: String,
    pub max_execution_time: i64,
    pub project_id: Option<String>,
    pub required_labels_json: String,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = tasks)]
pub struct NewTask {
    pub id: String,
    pub title: String,
    pub repo_url: String,
    pub description: String,
    pub base_branch: String,
    pub target_branch: String,
    pub status: String,
    pub priority: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub claimed_by: Option<String>,
    pub review_data_json: Option<String>,
    pub ssh_key: String,
    pub max_execution_time: i64,
    pub project_id: Option<String>,
    pub required_labels_json: String,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = tasks)]
pub struct TaskChangeset {
    pub title: Option<String>,
    pub repo_url: Option<String>,
    pub description: Option<String>,
    pub base_branch: Option<String>,
    pub target_branch: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub updated_at: Option<NaiveDateTime>,
    pub claimed_by: Option<Option<String>>,
    pub review_data_json: Option<Option<String>>,
    pub ssh_key: Option<String>,
    pub max_execution_time: Option<i64>,
    pub project_id: Option<Option<String>>,
    pub required_labels_json: Option<String>,
}

impl Task {
    pub fn get_uuid(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap_or_default()
    }

    pub fn get_claimed_by_uuid(&self) -> Option<Uuid> {
        self.claimed_by
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok())
    }

    pub fn get_project_id_uuid(&self) -> Option<Uuid> {
        self.project_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok())
    }
}
