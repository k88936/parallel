# AGENTS.md - Development Guide for AI Coding Agents

This document provides essential information for AI coding agents working in this repository.

## Project Overview

Parallel is a distributed system for managing AI coding agents that work on GitHub repositories. It consists of:
- **Server**: REST API + WebSocket server with SQLite task queue (Rust/Axum)
- **Worker(s)**: Long-running daemon processes that poll and execute tasks (Rust)
- **Web**: Next.js frontend for task management and human interaction

--

## Build/Lint/Test Commands

### Rust Backend

```bash
# Check for compilation errors (fast)
cargo check

# Build all binaries
cargo build --release

# Run all tests
cargo test

# Run a single test by name
cargo test test_full_task_lifecycle

# Run a single test in a specific file
cargo test --test integration_test test_worker_registration_and_task_claiming

# Run a specific binary
cargo run --bin server
cargo run --bin worker

```

### Web Frontend (in web/ directory)

```bash
cd web

# Development server
pnpm run dev

# Build for production
pnpm run build
```

#### Module Organization
```rust
// In mod.rs files, re-export public items
pub mod messages;
pub mod task;
pub mod worker;

pub use messages::*;
pub use task::*;
pub use worker::*;
```

#### Async Functions
- Use `#[tokio::test]` for async tests
- Use `async fn` for async handlers
- Prefer `tokio::spawn` for background tasks

#### Logging
```rust
// Use structured logging with fields
info!(task_id = %task_id, status = ?status, "Task status updated");

// Error logging
error!("Failed to create task: {}", e);
```

## Environment Variables

### Server
- `DATABASE_URL`: SQLite database URL (default: `sqlite://./data.db?mode=rwc`)
- `PORT`: Server port (default: 3000)

### Worker
- `SERVER_URL`: Server URL (default: `http://localhost:3000`)
- `WEBSOCKET_URL`: WebSocket URL (default: `ws://localhost:3000/ws/worker`)
- `WORKER_NAME`: Worker identifier (default: `worker-{hostname}`)
- `WORKER_WORK_BASE`: Working directory (default: `./work`)
- `MAX_CONCURRENT`: Max concurrent tasks (default: 4)

### Web
- `NEXT_PUBLIC_API_URL`: API base URL (default: `http://localhost:3000`)