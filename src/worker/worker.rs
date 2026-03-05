use crate::worker::client::TaskClient;
use crate::worker::git::GitOps;
use crate::worker::task::{Task, TaskResult};
use agent_client_protocol::{Agent as _, ClientCapabilities, FileSystemCapability, ContentBlock};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{error, info, warn};

pub struct Worker {
    work_base: PathBuf,
    agent_path: PathBuf,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
}

impl Worker {
    pub fn new(work_base: PathBuf, agent_path: PathBuf, max_concurrent: usize) -> Self {
        Self {
            work_base,
            agent_path,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn execute(&self, task: Task) -> TaskResult {
        let _permit = self.semaphore.acquire().await.unwrap();
        info!("Starting task {} in concurrent slot", task.id);

        let result = self.execute_task_inner(&task).await;

        match &result {
            Ok(()) => {
                info!("Task {} completed successfully", task.id);
                TaskResult::success(task.id.clone(), task.branch_name.clone())
            }
            Err(e) => {
                error!("Task {} failed: {}", task.id, e);
                TaskResult::failure(task.id.clone(), task.branch_name.clone(), e.to_string())
            }
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
                        ClientCapabilities::default()
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
