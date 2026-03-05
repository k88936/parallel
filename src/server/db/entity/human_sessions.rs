use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "human_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub attached_at: DateTimeUtc,
    pub permissions_json: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tasks::Entity",
        from = "Column::TaskId",
        to = "super::tasks::Column::Id"
    )]
    Task,
    #[sea_orm(
        belongs_to = "super::workers::Entity",
        from = "Column::WorkerId",
        to = "super::workers::Column::Id"
    )]
    Worker,
}

impl Related<super::tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<super::workers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Worker.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
