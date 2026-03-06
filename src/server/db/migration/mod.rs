pub use sea_orm_migration::prelude::*;

mod m20240305_000001_create_tasks;
mod m20240305_000002_create_workers;
mod m20240306_000003_update_workers_for_instructions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240305_000001_create_tasks::Migration),
            Box::new(m20240305_000002_create_workers::Migration),
            Box::new(m20240306_000003_update_workers_for_instructions::Migration),
        ]
    }
}