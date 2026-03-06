# AGENTS.md - Development Guide for AI Coding Agents

This document provides essential information for AI coding agents working in this repository.

## Project Overview

Parallel is a distributed system for managing AI coding agents that work on GitHub repositories. It consists of:
- **Server**: REST API server with SQLite task queue (Rust/Axum)
- **Worker(s)**: Long-running daemon processes that poll and execute tasks (Rust)
- **Web**: Next.js frontend for task management and human interaction

## Architecture

### Core Components

**Server** (`src/bin/server.rs`, `crates/server/`)
- REST API built with Axum framework
- SQLite database for persistent task queue
- Manages worker registration and instruction dispatch
- Endpoints: `/api/tasks/*`, `/api/workers/*`

**Worker** (`src/bin/worker.rs`, `crates/worker/`)
- Long-running daemon that polls server for tasks
- Executes AI coding agents in isolated work directories
- Handles Git operations (clone, branch, PR creation)
- Communicates with AI agent via Agent Client Protocol (ACP)

**Web UI** (`web/`)
- Next.js 16 frontend with React 19
- Tailwind CSS for styling
- Real-time task creation and monitoring
- Human review interface for task feedback

**Protocol** (`crates/protocol/`)
- Shared data structures and types
- Task definitions, status enums
- Worker-to-server communication protocol
- Instruction types for worker coordination

## File Navigation

### Root Level
```
/
├── Cargo.toml           # Workspace configuration with dependencies
├── data.db             # SQLite database (created at runtime)
├── work/               # Worker execution directories (UUID-based)
├── scripts/            # Test and deployment scripts
├── tests/              # Integration tests
├── src/bin/            # Binary entry points
│   ├── server.rs       # Server main()
│   └── worker.rs       # Worker main()
└── crates/             # Workspace crates
    ├── protocol/       # Shared types and protocol definitions
    ├── server/         # Server implementation
    └── worker/         # Worker implementation
```

### Server Crate (`crates/server/`)
```
crates/server/src/
├── lib.rs              # Router setup, server initialization
├── db/
│   ├── entity/         # SeaORM entity definitions
│   │   ├── tasks.rs    # Task table schema
│   │   └── workers.rs  # Worker table schema
│   └── migration/      # Database migrations
├── handlers/           # HTTP request handlers
│   ├── task.rs         # Task CRUD endpoints
│   └── worker.rs       # Worker registration & polling
├── services/           # Business logic layer
│   ├── coordinator.rs  # Task distribution logic
│   ├── task_service.rs # Task operations
│   └── worker_service.rs # Worker management
├── state.rs            # Application state (DB connection pool)
└── errors.rs           # Error types and handling
```

### Worker Crate (`crates/worker/`)
```
crates/worker/src/
├── lib.rs              # Public exports
├── worker.rs           # Main worker loop and registration
├── task.rs             # Task execution logic
├── agent_runner.rs     # AI agent process management
├── acp_client.rs       # Agent Client Protocol client
├── api_client.rs       # Server API client
└── repo_ops.rs         # Git operations (clone, branch, PR)
```

### Protocol Crate (`crates/protocol/`)
```
crates/protocol/src/
├── lib.rs              # Public exports
├── task.rs             # Task, TaskStatus, TaskPriority types
├── worker.rs           # Worker registration types
├── requests.rs         # API request structures
└── instructions.rs     # Worker instruction types
```

### Web Frontend (`web/`)
```
web/
├── package.json        # Dependencies: Next.js 16, React 19, Tailwind 4
├── src/
│   ├── app/
│   │   ├── layout.tsx  # Root layout
│   │   ├── page.tsx    # Home page (task list + create form)
│   │   └── tasks/[id]/
│   │       └── page.tsx # Task detail view
│   ├── components/
│   │   ├── TaskList.tsx      # Task list display
│   │   └── CreateTaskForm.tsx # Task creation form
│   ├── lib/
│   │   └── api.ts      # API client functions
│   └── types/
│       └── task.ts     # TypeScript type definitions
└── public/             # Static assets
```

## Data Flow

### Task Lifecycle
1. **Created** → User submits task via Web UI
2. **Queued** → Task enters server queue
3. **Claimed** → Worker claims task from queue
4. **InProgress** → Worker executes AI agent
5. **AwaitingReview** → Agent requests human feedback
6. **Reworking** → Worker incorporates feedback
7. **Completed** → Task finished successfully
8. **Cancelled/Failed** → Terminal error states

### Worker-Server Communication
1. Worker registers with server (`POST /api/workers/register`)
2. Worker polls for instructions (`POST /api/workers/poll`)
3. Server dispatches task assignments
4. Worker executes and reports events (`POST /api/workers/events`)
5. Worker updates task status (`POST /api/tasks/:id/status`)

## Key Types

### Task Status (`crates/protocol/src/task.rs:7`)
```rust
enum TaskStatus {
    Created, Queued, Claimed, InProgress,
    AwaitingReview, PendingRework, Completed,
    Cancelled, Failed,
}
```

### Task Priority (`crates/protocol/src/task.rs:51`)
```rust
enum TaskPriority {
    Low = 0, Normal = 1, High = 2, Urgent = 3,
}
```

### Task Structure (`crates/protocol/src/task.rs:82`)
```rust
struct Task {
    id: Uuid,
    repo_url: String,
    description: String,
    base_branch: String,
    target_branch: String,
    status: TaskStatus,
    priority: TaskPriority,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    claimed_by: Option<Uuid>,
    ssh_key: String,
}
```

## Development

### Running the Server
```bash
cargo run --bin server
# or with custom config:
DATABASE_URL="sqlite://./data.db?mode=rwc" PORT=3000 cargo run --bin server
```

### Running a Worker
```bash
cargo run --bin worker
# or with custom config:
SERVER_URL="http://localhost:3000" \
WORKER_NAME="worker-1" \
WORKER_WORK_BASE="./work" \
MAX_CONCURRENT=4 \
cargo run --bin worker
```

### Running the Web UI
```bash
cd web
npm install
npm run dev  # Runs on port 8080
```

### Testing
```bash
# Integration tests
cargo test

# Phase 1 end-to-end test
./scripts/test_phase1.sh
```

### Database Management
- SQLite database auto-created at `./data.db`
- Migrations run automatically on server startup
- Entity definitions: `crates/server/src/db/entity/`
- Migrations: `crates/server/src/db/migration/`

## Environment Variables

### Server
- `DATABASE_URL`: SQLite connection string (default: `sqlite://./data.db?mode=rwc`)
- `PORT`: Server port (default: 3000)

### Worker
- `SERVER_URL`: Server API endpoint (default: `http://localhost:3000`)
- `WORKER_NAME`: Unique worker identifier (default: `worker-{hostname}`)
- `WORKER_WORK_BASE`: Base directory for task execution (default: `./work`)
- `MAX_CONCURRENT`: Max concurrent tasks per worker (default: 4)

## API Endpoints

### Tasks
- `POST /api/tasks` - Create new task
- `GET /api/tasks` - List all tasks
- `GET /api/tasks/:id` - Get task details
- `DELETE /api/tasks/:id` - Cancel task
- `POST /api/tasks/:id/status` - Update task status
- `POST /api/tasks/:id/feedback` - Submit human feedback
- `GET /api/tasks/:id/review` - Get review data

### Workers
- `POST /api/workers/register` - Register new worker
- `POST /api/workers/poll` - Poll for instructions
- `POST /api/workers/events` - Push worker events
- `GET /api/workers` - List all workers

## Dependencies

### Rust (Workspace)
- **Web Framework**: Axum 0.7 with Tower middleware
- **Database**: SeaORM 1.0 with SQLite
- **Async Runtime**: Tokio 1.x
- **Serialization**: serde, serde_json
- **Logging**: tracing, tracing-subscriber
- **Date/Time**: chrono
- **Agent Protocol**: agent-client-protocol 0.9.5

### Web (Next.js)
- **Framework**: Next.js 16.1.6
- **UI**: React 19.2.3
- **Styling**: Tailwind CSS 4
- **Language**: TypeScript 5

## Common Patterns

### Adding a New API Endpoint
1. Add route in `crates/server/src/lib.rs`
2. Create handler in `crates/server/src/handlers/`
3. Add service logic in `crates/server/src/services/`
4. Define request/response types in `crates/protocol/src/`

### Adding a New Worker Instruction
1. Define instruction type in `crates/protocol/src/instructions.rs`
2. Handle instruction in `crates/worker/src/worker.rs`
3. Add instruction dispatch in `crates/server/src/services/coordinator.rs`

### Modifying Task Flow
1. Update status enum in `crates/protocol/src/task.rs`
2. Add migration if needed in `crates/server/src/db/migration/`
3. Update entity in `crates/server/src/db/entity/tasks.rs`
4. Update handlers and services as needed
5. Update Web UI types in `web/src/types/task.ts`

## Notes for AI Agents

- **Code Style**: Follow existing Rust conventions in the codebase
- **No Comments**: Do not add comments unless explicitly requested
- **Error Handling**: Use `anyhow::Result` for fallible operations
- **Async**: Use Tokio for all async operations
- **Database**: Always use SeaORM entities, never raw SQL
- **Testing**: Write integration tests in `tests/integration_test.rs`
- **Web UI**: Use TypeScript with strict typing
- **Commits**: Only commit when explicitly requested by user
