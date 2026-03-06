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
                    .table(Workers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Workers::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Workers::Name).string().not_null())
                    .col(ColumnDef::new(Workers::Status).string().not_null())
                    .col(
                        ColumnDef::new(Workers::LastHeartbeat)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Workers::CurrentTask).uuid())
                    .col(
                        ColumnDef::new(Workers::CapabilitiesJson)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Workers::MaxConcurrent).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_workers_current_task")
                            .from(Workers::Table, Workers::CurrentTask)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_workers_status")
                    .table(Workers::Table)
                    .col(Workers::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_workers_last_heartbeat")
                    .table(Workers::Table)
                    .col(Workers::LastHeartbeat)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Workers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Workers {
    Table,
    Id,
    Name,
    Status,
    LastHeartbeat,
    CurrentTask,
    CapabilitiesJson,
    MaxConcurrent,
}
