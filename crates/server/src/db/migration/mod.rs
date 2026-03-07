pub use sea_orm_migration::prelude::*;

mod m20240305_000001_create_tasks;
mod m20240305_000002_create_workers;
mod m20240306_000003_update_workers_for_instructions;
mod m20240306_000004_add_review_data_to_tasks;
mod m20240307_000005_add_ssh_key_to_tasks;
mod m20240308_000006_add_max_execution_time_to_tasks;
mod m20240309_000007_add_title_to_tasks;
mod m20240310_000008_add_token_to_workers;
mod m20240311_000009_create_projects;
mod m20240311_000010_add_project_id_to_tasks;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240305_000001_create_tasks::Migration),
            Box::new(m20240305_000002_create_workers::Migration),
            Box::new(m20240306_000003_update_workers_for_instructions::Migration),
            Box::new(m20240306_000004_add_review_data_to_tasks::Migration),
            Box::new(m20240307_000005_add_ssh_key_to_tasks::Migration),
            Box::new(m20240308_000006_add_max_execution_time_to_tasks::Migration),
            Box::new(m20240309_000007_add_title_to_tasks::Migration),
            Box::new(m20240310_000008_add_token_to_workers::Migration),
            Box::new(m20240311_000009_create_projects::Migration),
            Box::new(m20240311_000010_add_project_id_to_tasks::Migration),
        ]
    }
}
