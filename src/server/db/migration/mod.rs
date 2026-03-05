pub use sea_orm_migration::prelude::*;

mod m20240305_000001_create_tasks;
mod m20240305_000002_create_workers;
mod m20240305_000003_create_task_iterations;
mod m20240305_000004_create_human_sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240305_000001_create_tasks::Migration),
            Box::new(m20240305_000002_create_workers::Migration),
            Box::new(m20240305_000003_create_task_iterations::Migration),
            Box::new(m20240305_000004_create_human_sessions::Migration),
        ]
    }
}