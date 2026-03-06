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
│   ├── worker_service.rs # Worker management
│   ├── heartbeat_monitor.rs # Worker health monitoring
│   └── orphan_monitor.rs    # Orphan task detection
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

### Orphan Detection and Timeout Management
The server runs periodic background tasks to ensure task reliability:

**Orphan Detection**: Tasks in non-terminal states (InProgress, Claimed, AwaitingReview, PendingRework) with no active worker are automatically detected and re-queued.

**Timeout Management**: Tasks that exceed their `max_execution_time` are automatically marked as Failed. The timeout is measured from task creation.

Both monitors run at configurable intervals (see Environment Variables).