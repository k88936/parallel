# AGENTS.md - Development Guide for AI Coding Agents

This document provides essential information for AI coding agents working in this repository.

## Project Overview

Parallel is a distributed system for managing AI coding agents that work on GitHub repositories. It consists of:
- **Server**: REST API server with SQLite task queue (Rust/Axum)
- **Worker(s)**: Long-running daemon processes that poll and execute tasks (Rust)
- **Web**: Next.js frontend for task management and human interaction

---

## Build/Lint/Test Commands

### Rust Backend

```bash
# Check for compilation errors (fast)
cargo check
# Run a specific binary
cargo run --bin server
cargo run --bin worker

# Run all tests
cargo test

# Run a specific test
cargo test test_full_task_lifecycle
cargo test test_worker_poll_and_events

# Run a single test file
cargo test --test integration_test

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Lint code
cargo clippy

# Check for unused dependencies
cargo machete
```

### Web Frontend (in web/ directory)

```bash
cd web

# Install dependencies
pnpm install

# Development server
pnpm run dev

# Build for production
pnpm run build

# Lint code
pnpm run lint

# Type check
pnpm run type-check  # if available
```

---

## Code Style Guidelines

### Rust Code Style

**Error Handling**
- Use `anyhow::Result<T>` for fallible operations
- Use `.context()` to add error context: `.context("Failed to create task")?`
- Return `StatusCode` for HTTP handlers
- Log errors with `tracing::error!` before returning
- Use `anyhow::bail!` for early returns with errors

**Async Code**
- Use `tokio` runtime for async operations
- Mark async functions with `async fn`
- Use `.await` for async operations
- Use `tokio::spawn` for concurrent tasks
- Use `Arc<RwLock<T>>` for shared mutable state
- Use `mpsc` channels for message passing

**HTTP Handlers**
- Use `axum` extractors: `State`, `Path`, `Json`, `Query`
- Return `Result<Json<T>, StatusCode>` or `Result<StatusCode, StatusCode>`
- Use pattern matching for error handling

**Database Entities**
- Use Sea-ORM for database operations
- Define entities in `src/server/db/entity/`
- Use migrations in `src/server/db/migration/`
- Use `DateTimeUtc` for timestamps

---

### TypeScript/React Code Style (Web)

**TypeScript**
- Strict mode enabled
- Use `@/*` path aliases for imports
- Prefer functional components with hooks
- Use TypeScript interfaces for prop types

**Formatting**
- Use Next.js ESLint configuration
- Follow React best practices

---

## Architecture Reference

### Protocol Types (`src/protocol/`)
- `task.rs`: Task types, statuses, priorities
- `worker.rs`: Worker types and capabilities
- Instruction/Event pattern for worker communication

### Server Components (`src/server/`)
- `handlers/`: HTTP request handlers
- `db/`: Database entities and migrations
- `queue/`: Task scheduling logic
- `state.rs`: Shared application state

### Worker Components (`src/worker/`)
- `worker.rs`: Main worker loop and task execution
- `task.rs`: Task representation
- `git.rs`: Git operations
- `api_client.rs`: HTTP client for server communication
- `acp_client.rs`: Agent client protocol implementation

---

## Current Refactor Plan: Universal Poll/Event Architecture

### Phase 1: Protocol Redesign

**New Message Types in `src/protocol/`**

```rust
// Instructions (Server → Worker)
pub enum WorkerInstruction {
    AssignTask { task: Task },
    CancelTask { task_id: Uuid, reason: String },
    UpdateTask { task_id: Uuid, instruction: String },
    Pause { task_id: Uuid },
    Resume { task_id: Uuid },
}

// Events (Worker → Server)  
pub enum WorkerEvent {
    Heartbeat { running_tasks: Vec<Uuid> },
    TaskStarted { task_id: Uuid },
    TaskProgress { task_id: Uuid, message: String },
    TaskCompleted { task_id: Uuid },
    TaskFailed { task_id: Uuid, error: String },
    TaskCancelled { task_id: Uuid },
}
```

### Phase 2: Server Changes

**Files to modify:**

| File | Changes |
|------|---------|
| `protocol/task.rs` | Add `WorkerInstruction`, `WorkerEvent` types |
| `protocol/worker.rs` | Update `WorkerInfo` to track multiple tasks |
| `server/handlers/worker.rs` | Replace `heartbeat` + `claim_task` with `poll` + `events` handlers |
| `server/handlers/task.rs` | Remove `claim_task`, update `cancel_task` to queue instruction |
| `server/queue/scheduler.rs` | Add instruction queue per worker, task assignment logic |
| `server/server.rs` | Update routes |

**New API Endpoints:**
- `POST /api/worker/poll` - Long-poll for instructions
- `POST /api/worker/events` - Push events batch

### Phase 3: Worker Refactor

**Files to modify:**

| File | Changes |
|------|---------|
| `worker/worker.rs` | Complete rewrite with task management |
| `worker/task.rs` | Add `RunningTask` with cancellation token |
| `worker/api_client.rs` | Replace methods with `poll_instructions`, `send_events` |

**New Worker Architecture:**
```
Worker {
    running_tasks: HashMap<Uuid, RunningTask>,
    instruction_rx: mpsc::Receiver<WorkerInstruction>,
    event_tx: mpsc::Sender<WorkerEvent>,
}

RunningTask {
    task: Task,
    cancel_token: CancellationToken,
    instruction_tx: mpsc::Sender<TaskInstruction>,
}
```

**Worker Main Loop:**
1. Spawn `poll_loop` - continuously polls server for instructions
2. Spawn `event_loop` - batches and sends events to server
3. Main loop handles instructions:
   - `AssignTask` → spawn task executor with cancellation token
   - `CancelTask` → trigger cancellation, wait for graceful shutdown
   - `UpdateTask` → send instruction to running task's channel
