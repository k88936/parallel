use crate::server::queue::TaskScheduler;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub scheduler: Arc<TaskScheduler>,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        let scheduler = Arc::new(TaskScheduler::new(db.clone()));
        Self { db, scheduler }
    }
}
