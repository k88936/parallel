use anyhow::Result;
use axum::{
    routing::{get, post, delete},
    Router,
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::server::db::migration::Migrator;
use crate::server::handlers::{task, worker};
use crate::server::state::AppState;

pub async fn run_server(database_url: &str, port: u16) -> Result<()> {
    info!("Connecting to database: {}", database_url);
    let db = Database::connect(database_url).await?;

    info!("Running database migrations...");
    Migrator::up(&db, None).await?;

    let state = AppState::new(db);

    let app = Router::new()
        .route("/api/tasks", post(task::create_task))
        .route("/api/tasks", get(task::list_tasks))
        .route("/api/tasks/:id", get(task::get_task))
        .route("/api/tasks/:id", delete(task::cancel_task))
        .route("/api/tasks/:id/feedback", post(task::submit_feedback))
        .route("/api/tasks/:id/status", post(task::update_task_status))
        .route("/api/workers/register", post(worker::register_worker))
        .route("/api/workers/poll", post(worker::poll_instructions))
        .route("/api/workers/events", post(worker::push_events))
        .route("/api/workers", get(worker::list_workers))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
