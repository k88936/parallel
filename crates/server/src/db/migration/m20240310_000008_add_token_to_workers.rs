use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"ALTER TABLE workers ADD COLUMN token TEXT NOT NULL DEFAULT ''"#,
        )
        .await?;

        db.execute_unprepared(
            r#"CREATE UNIQUE INDEX idx_workers_token ON workers(token)"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(r#"DROP INDEX idx_workers_token"#)
            .await?;

        db.execute_unprepared(r#"ALTER TABLE workers DROP COLUMN token"#)
            .await?;

        Ok(())
    }
}
