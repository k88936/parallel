use sea_orm_migration::prelude::*;

use super::m20240305_000001_create_tasks::Tasks;
use super::m20240305_000002_create_workers::Workers;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HumanSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HumanSessions::SessionId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(HumanSessions::TaskId).uuid().not_null())
                    .col(ColumnDef::new(HumanSessions::WorkerId).uuid().not_null())
                    .col(ColumnDef::new(HumanSessions::AttachedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(HumanSessions::PermissionsJson).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_human_sessions_task")
                            .from(HumanSessions::Table, HumanSessions::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_human_sessions_worker")
                            .from(HumanSessions::Table, HumanSessions::WorkerId)
                            .to(Workers::Table, Workers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HumanSessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum HumanSessions {
    Table,
    SessionId,
    TaskId,
    WorkerId,
    AttachedAt,
    PermissionsJson,
}
