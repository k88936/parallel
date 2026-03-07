pub mod db;
pub mod errors;
pub mod error_codes;
pub mod api_error;
pub mod middleware;
pub mod handlers;
pub mod services;
pub mod state;

use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    routing::{delete, get, post},
    middleware::from_fn,
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tower_http::cors::CorsLayer;
use tower_http::request_id::SetRequestIdLayer;
use tracing::info;

use db::migration::Migrator;
use handlers::{task, worker};
use crate::middleware::{add_correlation_header, CorrelationIdGenerator};
use services::{
    Coordinator, EventProcessor, TaskService, WorkerService, spawn_heartbeat_monitor,
    spawn_orphan_monitor,
};
use state::AppState;

pub async fn run_server(database_url: &str, port: u16) -> Result<()> {
    info!("Connecting to database: {}", database_url);
    let db = Database::connect(database_url).await?;

    info!("Running database migrations...");
    Migrator::up(&db, None).await?;

    let task_service = Arc::new(TaskService::new(db.clone()));
    let worker_service = Arc::new(WorkerService::new(db.clone()));
    let coordinator = Arc::new(Coordinator::new(db.clone()));
    let event_processor = Arc::new(EventProcessor::new(
        task_service.clone(),
        worker_service.clone(),
    ));

    let state = AppState::new(
        task_service.clone(),
        worker_service.clone(),
        coordinator,
        event_processor,
    );

    let heartbeat_timeout: i64 = std::env::var("HEARTBEAT_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    let heartbeat_interval: u64 = std::env::var("HEARTBEAT_CHECK_INTERVAL_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let orphan_check_interval: u64 = std::env::var("ORPHAN_CHECK_INTERVAL_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    spawn_heartbeat_monitor(
        task_service.clone(),
        worker_service.clone(),
        heartbeat_timeout,
        heartbeat_interval,
    );
    spawn_orphan_monitor(task_service, worker_service, orphan_check_interval);

    let app = Router::new()
        .route("/api/tasks", post(task::create_task))
        .route("/api/tasks", get(task::list_tasks))
        .route("/api/tasks/:id", get(task::get_task))
        .route("/api/tasks/:id", delete(task::cancel_task))
        .route("/api/tasks/:id/feedback", post(task::submit_feedback))
        .route("/api/tasks/:id/review", get(task::get_review_data))
        .route("/api/tasks/:id/status", post(task::update_task_status))
        .route("/api/tasks/:id/retry", post(task::retry_task))
        .route("/api/workers/register", post(worker::register_worker))
        .route("/api/workers/poll", post(worker::poll_instructions))
        .route("/api/workers/events", post(worker::push_events))
        .route("/api/workers", get(worker::list_workers))
        .layer(from_fn(add_correlation_header))
        .layer(SetRequestIdLayer::new(
            axum::http::header::HeaderName::from_static("x-request-id"),
            CorrelationIdGenerator,
        ))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
