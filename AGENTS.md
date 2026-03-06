# AGENTS.md - Development Guide for AI Coding Agents

This document provides essential information for AI coding agents working in this repository.

## Project Overview

Parallel is a distributed system for managing AI coding agents that work on git repositories. It consists of:
- **Server**: REST API server with SQLite task queue (Rust/Axum)
- **Worker(s)**: Long-running daemon processes that poll and execute tasks (Rust)
- **Web**: Next.js frontend for task management and human interaction ( by now is a demo just for test)

```shell
# build
cargo build

# unit test (server)
cargo test --package parallel-server

```

* More detailed instructions (AGENT.md) are under a specific dir's root. *