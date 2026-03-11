# AGENTS.md - Development Guide for AI Coding Agents

This document provides essential information for AI coding agents working in this repository.

## Project Overview

Parallel is a distributed system for managing AI coding agents that work on git repositories. It consists of:
- **Server**: REST API server with SQLite task queue (Rust/Axum)
- **Worker(s)**: Long-running daemon processes that poll and execute tasks (Rust)
- **Web**: React/Vite frontend with Ring UI for task management and human interaction


```shell
# build
cargo build

```


## Codebase
* crates/common: common type define
* crates/message-broker: a tiny websocket-based message-broker
* crates/server: server proc
* crates/worker: worker proc
* parallel-web: React/Vite frontend with Ring UI

* More detailed instructions (AGENT.md) are under a specific dir's root.