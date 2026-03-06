use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub repo_url: String,
    pub description: String,
    pub base_branch: String,
    pub target_branch: String,
    pub status: String,
    pub priority: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub claimed_by: Option<Uuid>,
    pub review_data_json: Option<String>,
    pub ssh_key: String,
    pub max_execution_time: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::workers::Entity",
        from = "Column::ClaimedBy",
        to = "super::workers::Column::Id"
    )]
    Worker,
}

impl Related<super::workers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Worker.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
