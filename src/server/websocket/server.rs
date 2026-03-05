use axum::extract::ws::WebSocket;

pub async fn handle_worker_websocket(socket: WebSocket) {
    tracing::info!("Worker WebSocket connection established (not yet implemented)");
    let _ = socket;
}

pub async fn handle_human_websocket(socket: WebSocket) {
    tracing::info!("Human WebSocket connection established (not yet implemented)");
    let _ = socket;
}
