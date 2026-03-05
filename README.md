# Parallel - GitHub Agent Task System

A distributed system for managing AI coding agents that work on GitHub repositories.

## Architecture

- **Server**: REST API + WebSocket server with SQLite task queue
- **Worker(s)**: Long-running daemon processes that poll and execute tasks
- **Protocol**: Defined message types for communication

## Quick Start

### Start the Server

```bash
cargo run --bin server
```

The server will start on `http://localhost:3000` by default.

Environment variables:
- `DATABASE_URL`: SQLite database URL (default: `sqlite://./data.db?mode=rwc`)
- `PORT`: Server port (default: 3000)

### Run the Worker

```bash
cargo run --bin worker
```

## REST API Endpoints

### Task Management

#### Create Task
```bash
POST /api/tasks
Content-Type: application/json

{
  "repo_url": "git@github.com:user/repo.git",
  "description": "Add a hello world function",
  "base_branch": "main",          // optional, defaults to "main"
  "target_branch": "task/123",    // optional, auto-generated if not provided
  "priority": "normal"            // optional: "low", "normal", "high", "urgent"
}

Response: 200 OK
{
  "task_id": "uuid-here"
}
```

#### List Tasks
```bash
GET /api/tasks?status=queued&limit=10&offset=0

Response: 200 OK
{
  "tasks": [
    {
      "id": "uuid",
      "repo_url": "...",
      "description": "...",
      "base_branch": "main",
      "target_branch": "task/123",
      "status": "queued",
      "priority": "normal",
      "created_at": "2024-03-05T10:00:00Z",
      "updated_at": "2024-03-05T10:00:00Z",
      "claimed_by": null,
      "current_iteration": 0,
      "iterations": []
    }
  ],
  "total": 42
}
```

#### Get Task
```bash
GET /api/tasks/:id

Response: 200 OK
{
  "id": "uuid",
  "repo_url": "...",
  ...
}
```

#### Cancel Task
```bash
DELETE /api/tasks/:id

Response: 204 No Content
```

#### Claim Task (Worker)
```bash
POST /api/tasks/claim
Content-Type: application/json

{
  "worker_id": "worker-uuid"
}

Response: 200 OK
{
  "task": {
    "id": "uuid",
    "repo_url": "...",
    ...
  }
}
```

#### Submit Feedback
```bash
POST /api/tasks/:id/feedback
Content-Type: application/json

{
  "feedback_type": "request_changes",  // "approve", "request_changes", "abort"
  "message": "Please also add tests"
}

Response: 204 No Content
```

### Worker Management

#### Register Worker
```bash
POST /api/workers/register
Content-Type: application/json

{
  "name": "worker-01",
  "capabilities": {
    "has_git": true,
    "has_opencode": true,
    "supported_languages": ["rust", "python", "javascript"]
  },
  "max_concurrent": 4
}

Response: 200 OK
{
  "worker_id": "uuid"
}
```

#### Worker Heartbeat
```bash
POST /api/workers/heartbeat
Content-Type: application/json

{
  "worker_id": "worker-uuid",
  "current_task": "task-uuid"  // or null if idle
}

Response: 200 OK
{
  "acknowledged": true
}
```

#### List Workers
```bash
GET /api/workers

Response: 200 OK
[
  {
    "id": "uuid",
    "name": "worker-01",
    "status": "idle",  // "idle", "busy", "offline"
    "last_heartbeat": "2024-03-05T10:00:00Z",
    "current_task": null,
    "capabilities": {...},
    "max_concurrent": 4
  }
]
```

## Task Lifecycle

1. **Created** → Task submitted via API
2. **Queued** → Task ready for worker to claim
3. **Claimed** → Worker has claimed the task
4. **InProgress** → Worker executing via ACP
5. **AwaitingReview** → Worker finished, awaiting human review
6. **Iterating** → Human provided feedback, worker iterating
7. **Completed** → Finalized and pushed to branch
8. **Cancelled** → Task cancelled at any point

## Example Workflow

```bash
# 1. Create a task
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "repo_url": "git@github.com:user/test.git",
    "description": "Add a README file"
  }'

# Response: {"task_id": "abc-123"}

# 2. Check task status
curl http://localhost:3000/api/tasks/abc-123

# 3. Worker claims and executes task (handled automatically by worker binary)

# 4. Check all tasks
curl http://localhost:3000/api/tasks?status=queued

# 5. Provide feedback (when implemented)
curl -X POST http://localhost:3000/api/tasks/abc-123/feedback \
  -H "Content-Type: application/json" \
  -d '{
    "feedback_type": "approve",
    "message": "Looks good!"
  }'
```

## Database Schema

The system uses SQLite with SeaORM. Tables:
- `tasks`: Main task storage
- `task_iterations`: Tracks each iteration of a task
- `workers`: Worker registration and status
- `human_sessions`: Human attachment sessions

## Development Status

### Phase 1: Core Server & Database ✅
- [x] Protocol module with data structures
- [x] SQLite database with SeaORM
- [x] Task queue logic
- [x] REST API for task CRUD

### Phase 2: Worker Polling & Execution (Next)
- [ ] Worker HTTP client to server
- [ ] Polling logic
- [ ] Task claiming flow
- [ ] End-to-end execution

### Phase 3: WebSocket Streaming
- [ ] Worker WebSocket connection
- [ ] Real-time progress updates
- [ ] Agent output streaming

### Phase 4: Human Attachment
- [ ] Human WebSocket connection
- [ ] Real-time interaction during execution
- [ ] Terminal control

### Phase 5: Iteration & Review
- [ ] Task iteration logic
- [ ] Human feedback handling
- [ ] Complete workflow

## Building

```bash
# Check for errors
cargo check

# Build all binaries
cargo build --release

# Run tests (when available)
cargo test
```

## Dependencies

- **axum**: Web framework
- **SeaORM**: Async ORM for database
- **tokio**: Async runtime
- **serde**: Serialization
- **uuid**: UUID generation
- **chrono**: Date/time handling
