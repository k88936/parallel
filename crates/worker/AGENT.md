# AGENTS.md - Worker Crate

Long-running daemon that executes AI coding tasks by communicating with the server and running the opencode agent.

## Purpose

The worker is responsible for:
1. Registering with the server and maintaining a persistent token
2. Establishing WebSocket connection for real-time task assignment
3. Cloning repositories and preparing work directories
4. Running the `opencode acp` agent to implement tasks
5. Managing human-in-the-loop feedback cycles
6. Committing and pushing changes to target branches

## Architecture

```
src/
├── lib.rs              # Module exports
├── worker.rs           # Main worker loop and task orchestration
├── config.rs           # WorkerConfig (token persistence)
├── utils.rs            # Utility functions
├── conn/               # Connection utilities
│   ├── mod.rs
│   └── client_conn.rs
├── code/               # Task execution
│   ├── mod.rs
│   ├── task_runner.rs  # Runs opencode agent via ACP protocol
│   └── acp_client.rs   # Agent Client Protocol implementation
└── repo/               # Repository management
    ├── mod.rs
    ├── repo_ops.rs     # Git operations (clone, checkout, commit, push)
    └── repo_pool.rs    # Manages multiple repo slots for concurrent tasks
```

## Task Execution Flow

1. **Receive Assignment**: Worker gets `AssignTask` instruction via WebSocket
2. **Prepare Repository**: `RepoPool` acquires a slot, clones repo, creates target branch
3. **Run Agent**: `TaskRunner` spawns `opencode acp` process
4. **ACP Protocol**: Communicates with agent via stdin/stdout using Agent Client Protocol
5. **Human Review**: When agent completes, sends `TaskAwaitingReview` event
6. **Iteration Loop**: Waits for `ApproveIteration` or `ProvideFeedback` instructions
7. **Finalize**: On approval, commits and pushes changes to target branch

## Key Components

### Worker
- Maintains WebSocket connection with exponential backoff retry
- Sends periodic heartbeats with running task list
- Spawns separate tokio runtime per task for isolation
- Handles cancellation via `CancellationToken`

### TaskRunner
- Implements Agent Client Protocol (ACP) client
- Manages prompt-response cycle with agent
- Handles `StopReason::EndTurn` for review cycles
- Supports iteration based on human feedback

### RepoPool
- Manages concurrent access to multiple repositories
- Each task gets isolated working directory under `work_base/repos/<hash>/<task_id>`
- Handles SSH key injection for private repos
- Cleans up after task completion

## Worker Instructions

- `AssignTask { task }`: Start executing a task
- `CancelTask { task_id, reason }`: Cancel running task
- `ApproveIteration { task_id }`: Approve and finalize task
- `ProvideFeedback { task_id, feedback }`: Request iteration with feedback
- `AbortTask { task_id, reason }`: Force abort task

## Dependencies

- `tokio`: Async runtime
- `agent-client-protocol`: ACP for agent communication
- `git2`: Git operations
- `nix`: Signal handling (SIGTERM)
- `tokio-tungstenite`: WebSocket client
- `parallel-common`: Shared types
- `parallel-message-broker`: WebSocket client

## Configuration

Worker config is persisted to `<work_base>/worker_config.json`:
```json
{
  "token": "worker-auth-token"
}
```

This allows the worker to reconnect without re-registering.
