use crate::server::queue::TaskScheduler;
use crate::server::websocket::WebSocketState;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub scheduler: Arc<TaskScheduler>,
    pub ws_state: Arc<WebSocketState>,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        let scheduler = Arc::new(TaskScheduler::new(db.clone()));
        let ws_state = Arc::new(WebSocketState::new());
        Self {
            db,
            scheduler,
            ws_state,
        }
    }
}
