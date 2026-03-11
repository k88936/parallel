use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::projects;

pub const ROOT_PROJECT_ID: &str = "00000000-0000-0000-0000-000000000000";

pub fn is_root_project_id(id: &Uuid) -> bool {
    id.to_string() == ROOT_PROJECT_ID
}

#[derive(Queryable, Selectable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = projects)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repos_json: String,
    pub ssh_keys_json: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub parent_id: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = projects)]
pub struct NewProject {
    pub id: String,
    pub name: String,
    pub repos_json: String,
    pub ssh_keys_json: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub parent_id: Option<String>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = projects)]
pub struct ProjectChangeset {
    pub name: Option<String>,
    pub repos_json: Option<String>,
    pub ssh_keys_json: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub parent_id: Option<Option<String>>,
}

impl Project {
    pub fn get_uuid(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap_or_default()
    }

    pub fn get_parent_uuid(&self) -> Option<Uuid> {
        self.parent_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok())
    }

    pub fn is_root(&self) -> bool {
        self.id == ROOT_PROJECT_ID
    }
}
