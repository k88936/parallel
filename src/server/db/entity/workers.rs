use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "workers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub last_heartbeat: DateTimeUtc,
    pub current_task: Option<Uuid>,
    pub capabilities_json: String,
    pub max_concurrent: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tasks::Entity",
        from = "Column::CurrentTask",
        to = "super::tasks::Column::Id"
    )]
    Task,
}

impl Related<super::tasks::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
