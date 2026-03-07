pub mod db;
pub mod errors;
pub mod error_codes;
pub mod api_error;
pub mod middleware;
pub mod controller;
pub mod service;
pub mod state;
pub mod cron;
pub mod repository;

use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    routing::{delete, get, post, put},
    middleware::from_fn,
};
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tower_http::cors::CorsLayer;
use tower_http::request_id::SetRequestIdLayer;
use tracing::info;

use db::migration::Migrator;
use controller::{project, task, worker};
use crate::middleware::{add_correlation_header, CorrelationIdGenerator};
use parallel_message_broker::MessageBroker;
use service::{
    Coordinator, EventProcessor, ProjectService, TaskService, WorkerService,
    spawn_heartbeat_monitor, spawn_orphan_monitor, spawn_task_scheduler,
};
use state::AppState;

pub async fn run_server(database_url: &str, port: u16) -> Result<()> {
    info!("Connecting to database: {}", database_url);
    let db = Database::connect(database_url).await?;

    info!("Running database migrations...");
    Migrator::up(&db, None).await?;

    let task_service = Arc::new(TaskService::new(db.clone()));
    let worker_service = Arc::new(WorkerService::new(db.clone()));
    let project_service = Arc::new(ProjectService::new(db.clone()));
    let message_broker = MessageBroker::new();
    let coordinator = Arc::new(Coordinator::new(db.clone(), message_broker.clone()));
    let event_processor = Arc::new(EventProcessor::new(
        task_service.clone(),
        worker_service.clone(),
    ));

    let state = AppState::new(
        task_service.clone(),
        worker_service.clone(),
        project_service,
        coordinator.clone(),
        event_processor,
        message_broker.clone(),
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

    let task_scheduler_interval: u64 = std::env::var("TASK_SCHEDULER_INTERVAL_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    spawn_heartbeat_monitor(
        task_service.clone(),
        worker_service.clone(),
        heartbeat_timeout,
        heartbeat_interval,
    );
    spawn_orphan_monitor(task_service.clone(), worker_service.clone(), orphan_check_interval);
    spawn_task_scheduler(
        task_service,
        worker_service,
        coordinator,
        message_broker,
        task_scheduler_interval,
    );

    let app = Router::new()
        .route("/api/tasks", post(task::create_task))
        .route("/api/tasks", get(task::list_tasks))
        .route("/api/tasks/:id", get(task::get_task))
        .route("/api/tasks/:id", delete(task::cancel_task))
        .route("/api/tasks/:id/feedback", post(task::submit_feedback))
        .route("/api/tasks/:id/review", get(task::get_review_data))
        .route("/api/tasks/:id/status", post(task::update_task_status))
        .route("/api/tasks/:id/retry", post(task::retry_task))
        .route("/api/projects", post(project::create_project))
        .route("/api/projects", get(project::list_projects))
        .route("/api/projects/:id", get(project::get_project))
        .route("/api/projects/:id", put(project::update_project))
        .route("/api/projects/:id", delete(project::delete_project))
        .route("/api/workers/register", post(worker::register_worker))
        .route("/api/workers/ws", get(worker::worker_websocket))
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
