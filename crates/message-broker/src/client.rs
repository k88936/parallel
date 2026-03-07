use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::{Message as WsMessage, http::StatusCode}};
use tracing::{debug, error, info};

pub struct MessageBrokerClient {
    rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<String>,
    close_tx: mpsc::Sender<()>,
}

#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    Other(anyhow::Error),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Unauthorized => write!(f, "Unauthorized"),
            AuthError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AuthError {}

impl MessageBrokerClient {
    pub async fn connect_with_token(url: &str, token: &str) -> std::result::Result<Self, AuthError> {
        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(url)
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .map_err(|e| AuthError::Other(anyhow::anyhow!("Failed to build request: {}", e)))?;

        let (ws_stream, _response) = connect_async(request)
            .await
            .map_err(|e| {
                if let tokio_tungstenite::tungstenite::Error::Http(resp) = &e {
                    if resp.status() == StatusCode::UNAUTHORIZED {
                        return AuthError::Unauthorized;
                    }
                }
                AuthError::Other(anyhow::anyhow!("Failed to connect WebSocket: {}", e))
            })?;

        info!("WebSocket connected");

        let (ws_sink, ws_stream) = ws_stream.split();
        let (tx, rx) = mpsc::channel::<String>(64);
        let (out_tx, mut out_rx) = mpsc::channel::<String>(64);
        let (close_tx, mut close_rx) = mpsc::channel::<()>(1);

        let ws_sink = Arc::new(tokio::sync::Mutex::new(ws_sink));

        let ws_sink_clone = ws_sink.clone();
        tokio::spawn(async move {
            let mut ws_stream = ws_stream;
            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        let text = text.to_string();
                        debug!("Received message: {}", text);
                        if tx.send(text).await.is_err() {
                            break;
                        }
                    }
                    Ok(WsMessage::Ping(data)) => {
                        let _ = ws_sink_clone.lock().await.send(WsMessage::Pong(data)).await;
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!("WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        error!(error = %e, "WebSocket error");
                        break;
                    }
                    _ => {}
                }
            }
        });

        let ws_sink_clone = ws_sink.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(json) = out_rx.recv() => {
                        if ws_sink_clone.lock().await.send(WsMessage::text(json)).await.is_err() {
                            break;
                        }
                    }
                    _ = close_rx.recv() => {
                        let _ = ws_sink_clone.lock().await.send(WsMessage::Close(None)).await;
                        break;
                    }
                    else => break,
                }
            }
        });

        Ok(Self {
            rx,
            tx: out_tx,
            close_tx,
        })
    }

    pub async fn recv(&mut self) -> Option<String> {
        self.rx.recv().await
    }

    pub async fn send(&self, json: String) -> Result<()> {
        self.tx.send(json).await.context("Failed to send")
    }

    pub async fn close(&self) {
        let _ = self.close_tx.send(()).await;
    }
}
