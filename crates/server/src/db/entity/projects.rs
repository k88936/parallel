use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::schema::projects;

pub const ROOT_PROJECT_ID: &str = "root";

pub fn is_root_project_id(id: &str) -> bool {
    id == ROOT_PROJECT_ID
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
    pub fn is_root(&self) -> bool {
        self.id == ROOT_PROJECT_ID
    }
}
