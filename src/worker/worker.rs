use crate::protocol::{IterationResult, IterationStatus, Task as ProtocolTask, TaskStatus, WorkerCapabilities};
use crate::worker::task_client::TaskClient;
use crate::worker::git::GitOps;
use crate::worker::server_client::ServerClient;
use crate::worker::task::Task;
use agent_client_protocol::{Agent as _, ClientCapabilities as AcpClientCapabilities, FileSystemCapability, ContentBlock};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct Worker {
    work_base: PathBuf,
    agent_path: PathBuf,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
    server_client: Arc<ServerClient>,
    worker_id: Uuid,
    ssh_key_path: PathBuf,
}

impl Worker {
    pub fn new(
        work_base: PathBuf,
        agent_path: PathBuf,
        max_concurrent: usize,
        server_url: String,
        ssh_key_path: PathBuf,
    ) -> Self {
        Self {
            work_base,
            agent_path,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            server_client: Arc::new(ServerClient::new(server_url)),
            worker_id: Uuid::nil(),
            ssh_key_path,
        }
    }

    pub async fn register(&mut self, name: &str) -> Result<()> {
        let capabilities = WorkerCapabilities::default();
        
        let worker_info = self
            .server_client
            .register(name.to_string(), capabilities, self.max_concurrent)
            .await
            .context("Failed to register with server")?;

        self.worker_id = worker_info.id;
        info!("Worker registered with ID: {}", self.worker_id);

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        if self.worker_id == Uuid::nil() {
            anyhow::bail!("Worker not registered. Call register() first.");
        }

        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(10));
        let mut poll_interval = tokio::time::interval(Duration::from_secs(5));

        info!("Worker {} starting main loop", self.worker_id);

        let mut current_task: Option<Uuid> = None;

        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    if let Err(e) = self.server_client
                        .heartbeat(self.worker_id, current_task)
                        .await
                    {
                        warn!("Heartbeat failed: {}", e);
                    }
                }

                _ = poll_interval.tick() => {
                    if current_task.is_none() {
                        match self.try_claim_and_execute().await {
                            Ok(Some(task_id)) => {
                                current_task = Some(task_id);
                            }
                            Ok(None) => {
                                // No tasks available
                            }
                            Err(e) => {
                                error!("Error during task claim/execution: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn try_claim_and_execute(&self) -> Result<Option<Uuid>> {
        let task = self.server_client.claim_task(self.worker_id).await?;

        if let Some(protocol_task) = task {
            let task_id = protocol_task.id;
            info!("Claimed task: {}", task_id);

            let worker_task = self.protocol_to_worker_task(&protocol_task);

            let _permit = self.semaphore.acquire().await.unwrap();
            info!("Starting task {} in concurrent slot", task_id);

            let result = self.execute_task_inner(&worker_task).await;

            let (status, iteration_result) = match &result {
                Ok(()) => {
                    info!("Task {} completed successfully", task_id);
                    (
                        TaskStatus::Completed,
                        Some(IterationResult {
                            status: IterationStatus::Success,
                            summary: format!("Task completed: {}", worker_task.description),
                            files_changed: vec![],
                            commits: vec![format!("Implement: {}", worker_task.description)],
                            agent_messages: vec![],
                        }),
                    )
                }
                Err(e) => {
                    error!("Task {} failed: {}", task_id, e);
                    (
                        TaskStatus::Completed,
                        Some(IterationResult {
                            status: IterationStatus::Failed,
                            summary: format!("Task failed: {}", e),
                            files_changed: vec![],
                            commits: vec![],
                            agent_messages: vec![],
                        }),
                    )
                }
            };

            if let Err(e) = self.server_client
                .report_task_status(task_id, status, iteration_result)
                .await
            {
                error!("Failed to report task status: {}", e);
            }

            return Ok(Some(task_id));
        }

        Ok(None)
    }

    fn protocol_to_worker_task(&self, protocol_task: &ProtocolTask) -> Task {
        Task {
            id: protocol_task.id.to_string(),
            repo_url: protocol_task.repo_url.clone(),
            description: protocol_task.description.clone(),
            branch_name: protocol_task.target_branch.clone(),
            ssh_key_path: self.ssh_key_path.clone(),
        }
    }

    async fn execute_task_inner(&self, task: &Task) -> Result<()> {
        let task_dir = self.work_base.join(&task.id);
        tokio::fs::create_dir_all(&task_dir)
            .await
            .context("Failed to create task directory")?;

        let repo_dir = task_dir.join("repo");
        let git = GitOps::clone(&task.repo_url, &repo_dir, &task.ssh_key_path)?;
        git.create_branch(&task.branch_name)?;

        self.run_agent(&repo_dir, &task.description).await?;

        git.add_all()?;
        git.commit(&format!("Implement: {}", task.description))?;
        git.push(&task.branch_name, &task.ssh_key_path)?;

        if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
            warn!("Failed to cleanup task dir: {}", e);
        }

        Ok(())
    }

    async fn run_agent(&self, workdir: &PathBuf, prompt: &str) -> Result<()> {
        let workdir = std::fs::canonicalize(workdir)
            .context("Failed to resolve absolute path for workdir")?;
        
        info!("Starting agent in {:?}", workdir);

        let agent_path = which::which(&self.agent_path)
            .map_err(|_| anyhow::anyhow!("Agent binary not found: {:?}", self.agent_path))?;

        let mut child = tokio::process::Command::new(&agent_path)
            .args(["acp"])
            .current_dir(&workdir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn agent process")?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let client = TaskClient::new(workdir.clone());

        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async {
                let (conn, handle_io) =
                    agent_client_protocol::ClientSideConnection::new(client, stdin.compat_write(), stdout.compat(), |fut| {
                        tokio::task::spawn_local(fut);
                    });

                tokio::task::spawn_local(handle_io);

                conn.initialize(
                    agent_client_protocol::InitializeRequest::new(
                        agent_client_protocol::ProtocolVersion::LATEST
                    )
                    .client_capabilities(
                        AcpClientCapabilities::default()
                            .fs(FileSystemCapability::default()
                                .read_text_file(true)
                                .write_text_file(true))
                            .terminal(true)
                    )
                    .client_info(
                        agent_client_protocol::Implementation::new("parallel-worker", "0.1.0")
                    )
                )
                .await
                .context("Failed to initialize agent")?;

                let session = conn
                    .new_session(
                        agent_client_protocol::NewSessionRequest::new(workdir.clone())
                    )
                    .await
                    .context("Failed to create session")?;

                info!("Agent session created: {:?}", session.session_id);

                let text_content = agent_client_protocol::TextContent::new(prompt);
                let content_block = ContentBlock::Text(text_content);

                let result = conn
                    .prompt(
                        agent_client_protocol::PromptRequest::new(session.session_id, vec![content_block])
                    )
                    .await;


                match result {
                    Ok(response) => {
                        info!("Prompt completed with stop reason: {:?}", response.stop_reason);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Prompt failed: {}", e);
                        Err(anyhow::anyhow!("Prompt failed: {}", e))
                    }
                }
            })
            .await
    }
}
