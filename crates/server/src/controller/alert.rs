use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::Response,
};
use axum::extract::ws::Utf8Bytes;
use tracing::{error, info, warn};

use crate::state::AppState;

pub async fn alert_websocket(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_alert_websocket(socket, state))
}

async fn handle_alert_websocket(mut socket: WebSocket, state: AppState) {
    info!("Frontend WebSocket connected for alerts");

    let mut alert_rx = state.alert_service.subscribe();

    loop {
        tokio::select! {
            alert = alert_rx.recv() => {
                match alert {
                    Ok(payload) => {
                        let json = match serde_json::to_string(&payload) {
                            Ok(j) => j,
                            Err(e) => {
                                error!("Failed to serialize alert: {}", e);
                                continue;
                            }
                        };

                        if socket.send(Message::Text(Utf8Bytes::from(json))).await.is_err() {
                            warn!("Failed to send alert via WebSocket");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!("Alert channel closed");
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Alert WebSocket lagged behind by {} messages", n);
                    }
                }
            }

            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Frontend alert WebSocket closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("Alert WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
        }
    }

    info!("Frontend alert WebSocket disconnected");
}
