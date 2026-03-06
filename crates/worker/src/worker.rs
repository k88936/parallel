use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use parallel_protocol::{Task as ProtocolTask, WorkerCapabilities, WorkerEvent, WorkerInstruction};

use crate::api_client::APIClient;
use crate::repo_ops::GitOps;
use crate::repo_pool::RepoPool;
use crate::task_runner::TaskRunner;

use crate::task_runner::TaskInstruction;

struct RunningTask {
    cancel_token: CancellationToken,
    instruction_tx: mpsc::Sender<TaskInstruction>,
}

pub struct Worker {
    max_concurrent: usize,
    api_client: Arc<APIClient>,
    worker_id: uuid::Uuid,
    running_tasks: Arc<RwLock<HashMap<uuid::Uuid, RunningTask>>>,
    repo_pool: Arc<RepoPool>,
}

impl Worker {
    pub fn new(
        work_base: PathBuf,
        max_concurrent: usize,
        server_url: String,
    ) -> Self {
        let repo_pool_base = work_base.join("repos");
        let repo_pool = Arc::new(RepoPool::new(repo_pool_base));

        Self {
            max_concurrent,
            api_client: Arc::new(APIClient::new(server_url)),
            worker_id: uuid::Uuid::nil(),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            repo_pool,
        }
    }

    pub async fn register(&mut self, name: &str) -> Result<()> {
        let capabilities = WorkerCapabilities::default();

        let worker_info = self
            .api_client
            .register(name.to_string(), capabilities, self.max_concurrent)
            .await
            .context("Failed to register with server")?;

        self.worker_id = worker_info.id;
        info!("Worker registered with ID: {}", self.worker_id);

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        if self.worker_id == uuid::Uuid::nil() {
            anyhow::bail!("Worker not registered. Call register() first.");
        }

        info!("Worker {} starting main loop", self.worker_id);

        let worker_id = self.worker_id;
        let api_client = self.api_client.clone();
        let running_tasks = self.running_tasks.clone();

        let (event_tx, mut event_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(10));
            let mut pending_events: Vec<WorkerEvent> = Vec::new();

            loop {
                tokio::select! {
                    _ = heartbeat_interval.tick() => {
                        let tasks = running_tasks.read().await;
                        let running_task_ids: Vec<uuid::Uuid> = tasks.keys().copied().collect();
                        drop(tasks);

                        pending_events.push(WorkerEvent::Heartbeat {
                            running_tasks: running_task_ids
                        });
                    }

                    Some(event) = event_rx.recv() => {
                        pending_events.push(event);
                    }
                }

                if !pending_events.is_empty() {
                    let events_to_send = std::mem::take(&mut pending_events);

                    match api_client
                        .push_events(worker_id, events_to_send.clone())
                        .await
                    {
                        Ok(true) => {
                            pending_events.clear();
                        }
                        Ok(false) | Err(_) => {
                            pending_events = events_to_send;
                        }
                    }
                }
            }
        });

        let mut poll_interval = tokio::time::interval(Duration::from_secs(2));

        loop {
            tokio::select! {
                _ = poll_interval.tick() => {
                    match self.api_client.poll_instructions(self.worker_id).await {
                        Ok(instructions) => {
                            for instruction in instructions {
                                self.handle_instruction(instruction, event_tx.clone()).await;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to poll instructions: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn handle_instruction(
        &self,
        instruction: WorkerInstruction,
        event_tx: mpsc::Sender<WorkerEvent>,
    ) {
        match instruction {
            WorkerInstruction::AssignTask { task } => {
                let running = self.running_tasks.read().await;
                if running.len() >= self.max_concurrent {
                    warn!(
                        "Max concurrent tasks reached, cannot accept task {}",
                        task.id
                    );
                    return;
                }
                drop(running);

                let task_id = task.id;
                info!("Received task assignment: {}", task_id);

                let cancel_token = CancellationToken::new();
                let (instruction_tx, instruction_rx) = mpsc::channel(10);
                let running_task = RunningTask {
                    cancel_token: cancel_token.clone(),
                    instruction_tx,
                };

                {
                    let mut running = self.running_tasks.write().await;
                    running.insert(task_id, running_task);
                }

                let running_tasks = self.running_tasks.clone();
                let repo_pool = self.repo_pool.clone();

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
                            repo_pool,
                        )
                        .await;

                        {
                            let mut running = running_tasks.write().await;
                            running.remove(&task_id);
                        }

                        match result {
                            Ok(()) => {
                                info!("Task {} completed successfully", task_id);
                                let _ = event_tx.send(WorkerEvent::TaskCompleted { task_id }).await;
                            }
                            Err(e) => {
                                error!("Task {} failed: {}", task_id, e);
                                let _ = event_tx
                                    .send(WorkerEvent::TaskFailed {
                                        task_id,
                                        error: e.to_string(),
                                    })
                                    .await;
                            }
                        }
                    });
                });
            }
            WorkerInstruction::CancelTask { task_id, reason } => {
                info!("Received cancel for task {}: {}", task_id, reason);

                let running = self.running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    task.cancel_token.cancel();
                    info!("Sent cancellation signal to task {}", task_id);
                } else {
                    warn!("Task {} not found in running tasks", task_id);
                }
            }
            WorkerInstruction::UpdateTask {
                task_id,
                instruction,
            } => {
                info!("Received update for task {}: {}", task_id, instruction);
            }
            WorkerInstruction::ApproveIteration { task_id } => {
                info!("Received approve for task {}", task_id);
                let running = self.running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task.instruction_tx.send(TaskInstruction::Approve).await;
                } else {
                    warn!("Task {} not found for approval", task_id);
                }
            }
            WorkerInstruction::ProvideFeedback { task_id, feedback } => {
                info!("Received feedback for task {}", task_id);
                let running = self.running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task
                        .instruction_tx
                        .send(TaskInstruction::Iterate { feedback })
                        .await;
                } else {
                    warn!("Task {} not found for feedback", task_id);
                }
            }
            WorkerInstruction::AbortTask { task_id, reason } => {
                info!("Received abort for task {}: {}", task_id, reason);
                let running = self.running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task
                        .instruction_tx
                        .send(TaskInstruction::Abort { reason })
                        .await;
                } else {
                    warn!("Task {} not found for abort", task_id);
                }
            }
        }
    }

    async fn execute_task(
        task: &ProtocolTask,
        cancel_token: CancellationToken,
        instruction_rx: mpsc::Receiver<TaskInstruction>,
        event_tx: mpsc::Sender<WorkerEvent>,
        repo_pool: Arc<RepoPool>,
    ) -> Result<()> {

        let repo_dir = repo_pool
            .acquire_slot(
                &task.repo_url,
                task.id,
                &task.base_branch,
                &task.target_branch,
                &task.ssh_key,
            )
            .await
            .context("Failed to create task directory")?;

        if cancel_token.is_cancelled() {
            let _ = repo_pool.release_slot(&task.repo_url, task.id).await;
            return Err(anyhow::anyhow!("Task cancelled before execution"));
        }

        let workdir = std::fs::canonicalize(&repo_dir)
            .context("Failed to resolve absolute path for workdir")?;

        let runner = TaskRunner::new(task.id, task.description.clone(), workdir);

        runner
            .run(cancel_token.clone(), event_tx.clone(), instruction_rx)
            .await?;

        if !cancel_token.is_cancelled() {
            let git = GitOps::open(&repo_dir)?;
            git.add_all()?;
            git.commit(&format!("Implement: {}", task.description))?;
            git.push(&task.target_branch, &task.ssh_key)?;
        }

        if let Err(e) = repo_pool.release_slot(&task.repo_url, task.id).await {
            warn!("Failed to release repo slot: {}", e);
        }

        Ok(())
    }
}
