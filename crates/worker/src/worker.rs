use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use parallel_message_broker::MessageBrokerClient;
use parallel_common::{TaskAssignment, WorkerCapabilities, WorkerEvent, WorkerInfo, WorkerInstruction, RegisterWorkerRequest};

use crate::config::WorkerConfig;
use crate::repo::repo_pool::RepoPool;
use crate::code::task_runner::{TaskRunner, TaskInstruction};

struct RunningTask {
    cancel_token: CancellationToken,
    instruction_tx: mpsc::Sender<TaskInstruction>,
}

pub struct Worker {
    work_base: PathBuf,
    max_concurrent: usize,
    name: String,
    server_url: String,
    token: Option<String>,
    worker_id: uuid::Uuid,
    running_tasks: Arc<RwLock<HashMap<uuid::Uuid, RunningTask>>>,
    repo_pool: Arc<RepoPool>,
}

impl Worker {
    pub fn new(work_base: PathBuf, max_concurrent: usize, server_url: String) -> Self {
        let repo_pool_base = work_base.join("repos");
        let repo_pool = Arc::new(RepoPool::new(repo_pool_base));

        Self {
            work_base: work_base.clone(),
            max_concurrent,
            name: String::new(),
            server_url,
            token: None,
            worker_id: uuid::Uuid::nil(),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            repo_pool,
        }
    }

    pub async fn foo(&mut self, name: &str) -> Result<()> {
        self.name = name.to_string();

        if let Some(config) = WorkerConfig::load(&self.work_base)? {
            info!("Found existing worker config, validating stored token");
            self.token = Some(config.token);
        }

        if self.token.is_none() {
            self.register().await?;
        }

        Ok(())
    }

    async fn register(&mut self) -> Result<()> {
        let capabilities = WorkerCapabilities::default();
        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(60);

        loop {
            let url = format!("{}/api/workers/register", self.server_url);
            let request = RegisterWorkerRequest {
                name: self.name.clone(),
                capabilities: capabilities.clone(),
                max_concurrent: self.max_concurrent,
            };

            match reqwest::Client::new()
                .post(&url)
                .json(&request)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    let worker_info = response.json::<WorkerInfo>().await
                        .context("Failed to parse registration response")?;
                    
                    self.worker_id = worker_info.id;
                    self.token = Some(worker_info.token.clone());

                    let config = WorkerConfig::new(worker_info.token);
                    if let Err(e) = config.save(&self.work_base) {
                        warn!(error = %e, "Failed to save worker config");
                    }

                    info!(
                        worker_id = %self.worker_id,
                        worker_name = %self.name,
                        "Worker registered successfully"
                    );
                    return Ok(());
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    warn!(
                        status = %status,
                        body = %body,
                        retry_after_secs = delay.as_secs(),
                        "Registration failed, retrying"
                    );
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        retry_after_secs = delay.as_secs(),
                        "Registration request failed, retrying"
                    );
                }
            }

            tokio::time::sleep(delay).await;
            delay = std::cmp::min(delay * 2, max_delay);
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let token = self.token.clone().ok_or_else(|| {
            anyhow::anyhow!("Worker not registered. Call register() first.")
        })?;

        info!(
            worker_id = %self.worker_id,
            max_concurrent = self.max_concurrent,
            "Worker starting WebSocket connection"
        );

        let ws_url = self.server_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = format!("{}/api/workers/ws?token={}", ws_url, token);

        let mut connection = MessageBrokerClient::connect(&url).await
            .context("Failed to establish WebSocket connection")?;

        info!("WebSocket connected, waiting for instructions");

        let (event_tx, mut event_rx) = mpsc::channel::<WorkerEvent>(64);
        let running_tasks = self.running_tasks.clone();
        let repo_pool = self.repo_pool.clone();
        let worker_id = self.worker_id;

        let heartbeat_running_tasks = running_tasks.clone();
        let event_tx_heartbeat = event_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                let tasks = heartbeat_running_tasks.read().await;
                let running_task_ids: Vec<uuid::Uuid> = tasks.keys().copied().collect();
                drop(tasks);
                
                if event_tx_heartbeat.send(WorkerEvent::Heartbeat {
                    running_tasks: running_task_ids,
                }).await.is_err() {
                    break;
                }
            }
        });

        loop {
            tokio::select! {
                Some(json) = connection.recv() => {
                    match serde_json::from_str::<WorkerInstruction>(&json) {
                        Ok(instruction) => {
                            debug!(?instruction, "Received instruction");
                            Self::handle_instruction(
                                instruction,
                                running_tasks.clone(),
                                repo_pool.clone(),
                                event_tx.clone(),
                                worker_id,
                                self.max_concurrent,
                            ).await;
                        }
                        Err(e) => {
                            warn!(error = %e, json = %json, "Failed to parse instruction");
                        }
                    }
                }

                Some(event) = event_rx.recv() => {
                    let json = serde_json::to_string(&event)?;
                    if let Err(e) = connection.send(json).await {
                        error!(error = %e, "Failed to send event");
                    }
                }

                else => break,
            }
        }

        Ok(())
    }

    async fn handle_instruction(
        instruction: WorkerInstruction,
        running_tasks: Arc<RwLock<HashMap<uuid::Uuid, RunningTask>>>,
        repo_pool: Arc<RepoPool>,
        event_tx: mpsc::Sender<WorkerEvent>,
        worker_id: uuid::Uuid,
        max_concurrent: usize,
    ) {
        match instruction {
            WorkerInstruction::AssignTask { task } => {
                {
                    let running = running_tasks.read().await;
                    if running.len() >= max_concurrent {
                        warn!(
                            worker_id = %worker_id,
                            task_id = %task.id,
                            running_count = running.len(),
                            max_concurrent,
                            "Max concurrent tasks reached"
                        );
                        return;
                    }
                }

                let task_id = task.id;
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    repo_url = %task.repo_url,
                    "Received task assignment"
                );

                let cancel_token = CancellationToken::new();
                let (instruction_tx, instruction_rx) = mpsc::channel(10);
                let running_task = RunningTask {
                    cancel_token: cancel_token.clone(),
                    instruction_tx,
                };

                {
                    let mut running = running_tasks.write().await;
                    running.insert(task_id, running_task);
                }

                let running_tasks_clone = running_tasks.clone();
                let repo_pool_clone = repo_pool.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create tokio runtime");

                    rt.block_on(async {
                        let _ = event_tx.send(WorkerEvent::TaskStarted { task_id }).await;

                        let result = Self::execute_task(
                            &task,
                            cancel_token,
                            instruction_rx,
                            event_tx.clone(),
                            repo_pool_clone,
                        ).await;

                        {
                            let mut running = running_tasks_clone.write().await;
                            running.remove(&task_id);
                        }

                        match result {
                            Ok(()) => {
                                info!(task_id = %task_id, "Task completed");
                                let _ = event_tx.send(WorkerEvent::TaskCompleted { task_id }).await;
                            }
                            Err(e) => {
                                error!(task_id = %task_id, error = %e, "Task failed");
                                let _ = event_tx.send(WorkerEvent::TaskFailed {
                                    task_id,
                                    error: e.to_string(),
                                }).await;
                            }
                        }
                    });
                });
            }
            WorkerInstruction::CancelTask { task_id, reason } => {
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    reason = %reason,
                    "Received cancel request"
                );

                let running = running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    task.cancel_token.cancel();
                }
            }
            WorkerInstruction::UpdateTask { task_id, instruction } => {
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    instruction = %instruction,
                    "Received update"
                );
            }
            WorkerInstruction::ApproveIteration { task_id } => {
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    "Received approval"
                );
                let running = running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task.instruction_tx.send(TaskInstruction::Approve).await;
                }
            }
            WorkerInstruction::ProvideFeedback { task_id, feedback } => {
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    feedback_type = ?feedback.feedback_type,
                    "Received feedback"
                );
                let running = running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task.instruction_tx.send(TaskInstruction::Iterate { feedback }).await;
                }
            }
            WorkerInstruction::AbortTask { task_id, reason } => {
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    reason = %reason,
                    "Received abort"
                );
                let running = running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task.instruction_tx.send(TaskInstruction::Abort { reason }).await;
                }
            }
        }
    }

    async fn execute_task(
        task: &TaskAssignment,
        cancel_token: CancellationToken,
        instruction_rx: mpsc::Receiver<TaskInstruction>,
        event_tx: mpsc::Sender<WorkerEvent>,
        repo_pool: Arc<RepoPool>,
    ) -> Result<()> {
        info!(
            task_id = %task.id,
            repo_url = %task.repo_url,
            "Starting task execution"
        );

        let repo_dir = repo_pool
            .acquire_slot(
                &task.repo_url,
                task.id,
                &task.base_branch,
                &task.target_branch,
                &task.ssh_key,
            )
            .await
            .context("Failed to get task directory")?;

        if cancel_token.is_cancelled() {
            info!(task_id = %task.id, "Task cancelled before execution");
            let _ = repo_pool.release_slot(&task.repo_url, task.id).await;
            return Err(anyhow::anyhow!("Task cancelled before execution"));
        }

        let workdir = std::fs::canonicalize(&repo_dir)
            .context("Failed to resolve absolute path for workdir")?;

        debug!(
            task_id = %task.id,
            workdir = ?workdir,
            "Task workdir prepared"
        );

        let runner = TaskRunner::new(task.id, task.description.clone(), workdir);

        runner
            .run(cancel_token.clone(), event_tx.clone(), instruction_rx)
            .await?;

        if !cancel_token.is_cancelled() {
            info!(
                task_id = %task.id,
                target_branch = %task.target_branch,
                "Committing and pushing changes"
            );

            use crate::repo::repo_ops::GitOps;
            let git = GitOps::open(&repo_dir)?;
            git.add_all()?;
            git.commit(&format!("Implement: {}", task.description))?;
            git.push(&task.target_branch, &task.ssh_key)?;

            info!(
                task_id = %task.id,
                target_branch = %task.target_branch,
                "Changes pushed successfully"
            );
        }

        if let Err(e) = repo_pool.release_slot(&task.repo_url, task.id).await {
            warn!(task_id = %task.id, error = %e, "Failed to release repo slot");
        }

        Ok(())
    }
}
