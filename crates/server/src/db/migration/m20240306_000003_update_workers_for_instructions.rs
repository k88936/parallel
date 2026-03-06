use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"ALTER TABLE workers ADD COLUMN current_tasks_json TEXT NOT NULL DEFAULT '[]'"#,
        )
        .await?;

        db.execute_unprepared(
            r#"ALTER TABLE workers ADD COLUMN pending_instructions_json TEXT NOT NULL DEFAULT '[]'"#
        ).await?;

        db.execute_unprepared(
            r#"UPDATE workers SET current_tasks_json = json_array(current_task) WHERE current_task IS NOT NULL"#
        ).await?;

        db.execute_unprepared(
            r#"UPDATE workers SET current_tasks_json = '[]' WHERE current_task IS NULL"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(r#"ALTER TABLE workers DROP COLUMN current_tasks_json"#)
            .await?;

        db.execute_unprepared(r#"ALTER TABLE workers DROP COLUMN pending_instructions_json"#)
            .await?;

        Ok(())
    }
}
