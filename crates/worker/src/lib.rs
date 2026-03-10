mod actors;
mod code;
mod config;
mod repo;
pub mod utils;

use crate::actors::manager::{GetRunningTaskIds, HandleInstruction};
use crate::actors::{ManagerActor, RepoPoolActor};
use anyhow::{Context, Error};
pub use config::{AcpConfig, WorkerConfig};
use parallel_common::{
    RegisterWorkerRequest, WorkerCapabilities, WorkerEvent, WorkerInfo, WorkerInstruction,
};
use parallel_message_broker::{AuthError, MessageBrokerClient};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use xtra::{Address, Mailbox};
use axum::{Router, routing::get, http::StatusCode, Json};
use serde_json::json;
use tower_http::cors::CorsLayer;

pub struct Config {
    pub work_base: PathBuf,
    pub max_concurrent: usize,
    pub server_url: String,
    pub name: String,
    pub health_port: u16,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            work_base: "./work".into(),
            max_concurrent: 4,
            server_url: "localhost:3000".into(),
            name: "worker".to_string(),
            health_port: 8080,
        }
    }
}
impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            work_base: self.work_base.clone(),
            max_concurrent: self.max_concurrent,
            server_url: self.server_url.clone(),
            name: self.name.clone(),
            health_port: self.health_port,
        }
    }
}
pub struct App {
    config: Config,
    acp_config: AcpConfig,
}

impl App {
    pub fn new(config: Config) -> Self {
        let acp_config = AcpConfig::load(&config.work_base).unwrap_or_else(|e| {
            tracing::warn!("Failed to load acp config: {}, using empty config", e);
            AcpConfig {
                agent_servers: Default::default(),
            }
        });
        Self { config, acp_config }
    }
    pub async fn run(&self) {
        let health_addr: SocketAddr = format!("0.0.0.0:{}", self.config.health_port).parse().unwrap();
        let health_app = Router::new()
            .route("/health", get(health_check))
            .layer(CorsLayer::permissive());

        tracing::info!("Worker health check listening on {}", health_addr);
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(health_addr).await.unwrap();
            axum::serve(listener, health_app).await.unwrap();
        });

        let repo_pool_base = self.config.work_base.join("repos");
        let repo_pool_addr =
            xtra::spawn_tokio(RepoPoolActor::new(repo_pool_base), Mailbox::unbounded());

        let (manager_addr, manager_mailbox) = Mailbox::unbounded();
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(64);
        xtra::spawn_tokio(
            ManagerActor::new(
                self.config.clone(),
                self.acp_config.clone(),
                repo_pool_addr,
                manager_addr.clone(),
                event_tx.clone(),
            ),
            (manager_addr.clone(), manager_mailbox),
        );

        self.connect(manager_addr.clone(), event_tx, event_rx).await.unwrap();
    }
    async fn connect(
        &self,
        manager_addr: Address<ManagerActor>,
        event_tx: Sender<WorkerEvent>,
        mut event_rx: Receiver<WorkerEvent>,
    ) -> anyhow::Result<()> {
        let capabilities = WorkerCapabilities {
            has_git: true,
            available_agents: self.acp_config.available_agents(),
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
            ],
        };

        let mut token = None;

        if let Some(config) = WorkerConfig::load(&self.config.work_base)? {
            tracing::info!("Found existing worker config, validating stored token");
            token = Some(config.token);
        }

        loop {
            if token.is_none() {
                self.register(capabilities.clone(), &mut token).await?;
            }

            let ws_url = self
                .config
                .server_url
                .replace("http://", "ws://")
                .replace("https://", "wss://");
            let url = format!("{}/api/workers/ws", ws_url);

            match MessageBrokerClient::connect_with_token(&url, token.as_ref().unwrap()).await {
                Ok(mut client) => {
                    tracing::info!("WebSocket connected, waiting for instructions");
                    tokio::spawn(Self::heartbeat(manager_addr.clone(), event_tx.clone()));
                    loop {
                        tokio::select! {
                            // receive inst from server
                            Some(json) = client.recv() => {
                                Self::forward_inst(json,manager_addr.clone()).await
                            }
                            // report worker event
                            Some(event) = event_rx.recv() => {
                                let json = serde_json::to_string(&event)?;
                                if client.send(json).await.is_err() {
                                    break;
                                }
                            }
                            else => break,
                        }
                    }
                }
                Err(AuthError::Unauthorized) => {
                    token = None;
                    continue;
                }
                Err(AuthError::Other(e)) => {
                    tracing::error!(error = %e, "WebSocket connection failed, retrying in 5s");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }
        }
    }

    async fn register(
        &self,
        capabilities: WorkerCapabilities,
        token: &mut Option<String>,
    ) -> Result<(), Error> {
        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);

        loop {
            let url = format!("{}/api/workers/register", self.config.server_url);
            let name = hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "worker".to_string());

            let request = RegisterWorkerRequest {
                name,
                capabilities: capabilities.clone(),
                max_concurrent: self.config.max_concurrent,
            };

            match reqwest::Client::new()
                .post(&url)
                .json(&request)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    let worker_info = response
                        .json::<WorkerInfo>()
                        .await
                        .context("Failed to parse registration response")?;

                    *token = Some(worker_info.token.clone());

                    let config = WorkerConfig::new(worker_info.token);
                    if let Err(e) = config.save(&self.config.work_base) {
                        tracing::warn!(error = %e, "Failed to save worker config");
                    }

                    tracing::info!("Worker registered successfully");
                    break;
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    tracing::warn!(
                        status = %status,
                        body = %body,
                        retry_after_secs = delay.as_secs(),
                        "Registration failed, retrying"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        retry_after_secs = delay.as_secs(),
                        "Registration request failed, retrying"
                    );
                }
            }

            tokio::time::sleep(delay).await;
            delay = std::cmp::min(delay * 2, max_delay);
        }
        Ok(())
    }

    async fn heartbeat(manager_addr: Address<ManagerActor>, event_tx: Sender<WorkerEvent>) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            match manager_addr.clone().send(GetRunningTaskIds).await {
                Ok(running_task_ids) => {
                    let event = WorkerEvent::Heartbeat {
                        running_tasks: running_task_ids,
                    };
                    let _ = event_tx.send(event).await;
                }
                Err(_) => break,
            }
        }
    }
    async fn forward_inst(json: String, manager_addr: Address<ManagerActor>) {
        match serde_json::from_str::<WorkerInstruction>(&json) {
            Ok(instruction) => {
                tracing::debug!(?instruction, "Received instruction");
                manager_addr
                    .send(HandleInstruction(instruction))
                    .await
                    .unwrap();
            }
            Err(e) => {
                tracing::warn!(error = %e, json = %json, "Failed to parse instruction");
            }
        }
    }
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}
