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
# Run a specific binary
cargo run --bin server
cargo run --bin worker

```

### Web Frontend (in web/ directory)

```bash
cd web

# Development server
pnpm run dev
```