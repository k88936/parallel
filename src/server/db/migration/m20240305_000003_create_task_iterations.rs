use sea_orm_migration::prelude::*;

use super::m20240305_000001_create_tasks::Tasks;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TaskIterations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskIterations::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskIterations::TaskId).uuid().not_null())
                    .col(ColumnDef::new(TaskIterations::IterationId).integer().not_null())
                    .col(ColumnDef::new(TaskIterations::StartedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TaskIterations::CompletedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(TaskIterations::ResultJson).string())
                    .col(ColumnDef::new(TaskIterations::HumanFeedbackJson).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_iterations_task")
                            .from(TaskIterations::Table, TaskIterations::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TaskIterations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TaskIterations {
    Table,
    Id,
    TaskId,
    IterationId,
    StartedAt,
    CompletedAt,
    ResultJson,
    HumanFeedbackJson,
}
