# AGENTS.md - Common Crate

Shared type definitions and utilities used across all crates in the Parallel system.

## Purpose

This crate provides common data structures, enums, and request/response types that are shared between the server, worker, and message-broker components. It serves as the foundational layer for inter-component communication.

## Structure

```
src/
├── lib.rs           # Module exports
├── task.rs          # Task types (TaskStatus, TaskPriority, Task)
├── worker.rs        # Worker types (WorkerStatus, WorkerCapabilities, WorkerInfo)
├── project.rs       # Project types
├── instructions.rs  # WorkerInstruction, WorkerEvent enums for server-worker communication
└── requests.rs      # API request types (RegisterWorkerRequest, HumanFeedback, etc.)
```

## Key Types

### Task Lifecycle
- `TaskStatus`: Created → Queued → Claimed → InProgress → AwaitingReview → PendingResponse → Completed/Cancelled/Failed
- `TaskPriority`: Low, Normal, High, Urgent

### Worker Lifecycle
- `WorkerStatus`: Idle, Busy, Offline, Dead
- `WorkerCapabilities`: Describes worker's tools (git, opencode, languages)
- `WorkerInfo`: Worker identity and state

### Communication
- `WorkerInstruction`: Server → Worker (AssignTask, CancelTask, ApproveIteration, ProvideFeedback, etc.)
- `WorkerEvent`: Worker → Server (Heartbeat, TaskStarted, TaskCompleted, TaskAwaitingReview, etc.)

## Dependencies

- `serde`, `serde_json`: Serialization
- `uuid`: Unique identifiers
- `chrono`: Timestamps

## Usage

Add to Cargo.toml:
```toml
parallel-common = { path = "crates/common" }
```

Import types:
```rust
use parallel_common::{Task, TaskStatus, WorkerInstruction, WorkerEvent};
```
