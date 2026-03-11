# AGENTS.md - Server Crate

REST API server with SQLite database for managing AI coding agent tasks and workers.

## Purpose

The server provides the central coordination point for the Parallel system. It exposes a REST API for task submission, manages worker registration and heartbeat monitoring, and orchestrates task scheduling and distribution to workers via WebSocket.

## Architecture

```
src/
├── lib.rs              # Router setup and server entry point
├── state.rs            # AppState with service dependencies
├── controller/         # HTTP request handlers (thin layer)
│   ├── task.rs         # Task CRUD endpoints
│   ├── worker.rs       # Worker registration and WebSocket
│   └── project.rs      # Project CRUD endpoints
├── service/            # Business logic layer
│   ├── task_service.rs     # Task state machine and operations
│   ├── worker_service.rs   # Worker management
│   ├── project_service.rs  # Project management
│   └── worker_event_service.rs  # WebSocket event handling
├── repository/         # Data access layer (SeaORM)
│   ├── task_repository.rs
│   ├── worker_repository.rs
│   └── project_repository.rs
├── cron/               # Background jobs
│   ├── heartbeat_monitor.rs   # Detect offline workers
│   ├── orphan_monitor.rs      # Handle orphaned tasks
│   └── task_scheduler.rs      # Assign tasks to workers
├── db/                 # Database layer
│   ├── entity/         # SeaORM entity definitions
│   └── migration/      # Database migrations
├── errors.rs           # Error types
├── error_codes.rs      # API error codes
├── api_error.rs        # API error response format
├── middleware.rs       # Request ID and correlation headers
└── common/             # Server-specific types
```

## API Endpoints

### Tasks
- `POST /api/tasks` - Create task
- `GET /api/tasks` - List tasks
- `GET /api/tasks/:id` - Get task details
- `DELETE /api/tasks/:id` - Cancel task
- `POST /api/tasks/:id/feedback` - Submit human feedback
- `GET /api/tasks/:id/review` - Get review data
- `POST /api/tasks/:id/status` - Update status
- `POST /api/tasks/:id/retry` - Retry failed task

### Workers
- `POST /api/workers/register` - Register new worker
- `GET /api/workers/ws` - WebSocket connection
- `GET /api/workers` - List workers

### Projects
- `POST /api/projects` - Create project
- `GET /api/projects` - List projects
- `GET /api/projects/:id` - Get project
- `PUT /api/projects/:id` - Update project
- `DELETE /api/projects/:id` - Delete project

## Background Jobs

1. **Heartbeat Monitor**: Marks workers as offline if no heartbeat received within timeout
2. **Orphan Monitor**: Re-queues tasks left by dead/offline workers
3. **Task Scheduler**: Polls for queued tasks and assigns to available workers

## Dependencies

- `axum`: Web framework
- `sea-orm`: Database ORM (SQLite)
- `tower-http`: Middleware (CORS, request ID)
- `parallel-common`: Shared types
- `parallel-message-broker`: WebSocket messaging

## Environment Variables

- `HEARTBEAT_TIMEOUT_SECONDS`: Worker timeout (default: 30)
- `HEARTBEAT_CHECK_INTERVAL_SECONDS`: Check interval (default: 10)
- `ORPHAN_CHECK_INTERVAL_SECONDS`: Orphan check interval (default: 60)
- `TASK_SCHEDULER_INTERVAL_SECONDS`: Scheduler interval (default: 2)


## to use diesel cli
```shell
cd current-dir
export DATABASE_URL="sqlite://./data.db?mode=rwc" diesel ...
```