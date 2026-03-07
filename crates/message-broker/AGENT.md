# AGENTS.md - Message Broker Crate

Lightweight WebSocket-based message broker for real-time communication between server and workers.

## Purpose

Provides a simple pub/sub message broker that enables bidirectional communication between the server and connected workers. Uses WebSocket connections for real-time task assignment and event streaming.

## Architecture

```
src/
├── lib.rs      # Module exports
├── server.rs   # MessageBrokerServer, WorkerChannel
└── client.rs   # MessageBrokerClient
```

## Components

### MessageBrokerServer

In-memory message broker used by the server:

```rust
let broker = MessageBrokerServer::new();

// Register a worker's message channel
broker.register(worker_id);

// Send instruction to specific worker
broker.send(&worker_id, json_string);

// Subscribe to worker's events
let rx = broker.subscribe(&worker_id);

// Check if worker is connected
broker.is_connected(&worker_id);

// Get list of connected workers
broker.connected_ids();

// Remove worker on disconnect
broker.unregister(&worker_id);
```

### WorkerChannel

Per-worker broadcast channel:
- Uses `tokio::sync::broadcast` for fan-out messaging
- Default capacity: 256 messages
- Multiple subscribers can receive same messages

### MessageBrokerClient

WebSocket client used by workers:

```rust
let mut client = MessageBrokerClient::connect(&ws_url).await?;

// Receive messages from server
while let Some(json) = client.recv().await {
    // Handle instruction
}

// Send events to server
client.send(json_string).await?;

// Close connection
client.close().await;
```

## Message Flow

```
Server                          Broker                         Worker
  |                               |                               |
  |--- AssignTask -------------->|                               |
  |                               |--- WebSocket message -------->|
  |                               |                               |
  |                               |<-- WorkerEvent (Heartbeat) ---|
  |<-- EventProcessor receives ---|                               |
  |                               |                               |
  |                               |<-- TaskCompleted -------------|
  |<-- EventProcessor receives ---|                               |
```

## Design Decisions

1. **Broadcast Channels**: Enables multiple handlers per worker (e.g., logging + processing)
2. **Arc<String>**: Avoids string cloning for broadcast messages
3. **Separate Client/Server**: Clean separation for different deployment contexts
4. **Simple Protocol**: JSON strings over WebSocket, no framing protocol

## Dependencies

- `tokio`: Async runtime, broadcast channels
- `tokio-tungstenite`: WebSocket (client only)
- `dashmap`: Concurrent map for server-side channel storage
- `futures`: Stream/Sink traits

## Usage

### Server-side

```rust
use parallel_message_broker::MessageBrokerServer;

let broker = MessageBrokerServer::new();
broker.register(worker_id);

// In WebSocket handler:
broker.subscribe(&worker_id)

// To send instruction:
broker.send(&worker_id, serde_json::to_string(&instruction)?)
```

### Worker-side

```rust
use parallel_message_broker::MessageBrokerClient;

let mut conn = MessageBrokerClient::connect(&ws_url).await?;

loop {
    tokio::select! {
        Some(json) = conn.recv() => {
            // Handle server instruction
        }
        Some(event) = event_rx.recv() => {
            conn.send(serde_json::to_string(&event)?).await?;
        }
    }
}
```
