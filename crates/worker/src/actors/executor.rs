use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tokio_util::sync::CancellationToken;
use xtra::{Actor, Address, Mailbox};

use agent_client_protocol as acp;
use agent_client_protocol::{
    Agent, ClientCapabilities as AcpClientCapabilities, ContentBlock, FileSystemCapabilities,
};

use parallel_common::{HumanFeedback, TaskAssignment, WorkerEvent};

use crate::code::acp_client::ACPClient;
use crate::repo::repo_ops::GitOps;
use crate::actors::{RepoPoolActor, AcquireSlot, ReleaseSlot, TaskCompleted, ManagerActor};
use crate::AcpConfig;

#[derive(Debug, Clone)]
pub enum TaskInstruction {
    Approve,
    Iterate { feedback: HumanFeedback },
    Abort { reason: String },
}

pub struct ExecutorActor {
    task: TaskAssignment,
    cancel_token: CancellationToken,
    instruction_tx: tokio::sync::mpsc::Sender<TaskInstruction>,
    instruction_rx: Option<tokio::sync::mpsc::Receiver<TaskInstruction>>,
    repo_pool: Address<RepoPoolActor>,
    event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
    worker: Address<ManagerActor>,
    acp_config: AcpConfig,
}

impl ExecutorActor {
    pub fn new(
        task: TaskAssignment,
        repo_pool: Address<RepoPoolActor>,
        event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
        worker: Address<ManagerActor>,
        acp_config: AcpConfig,
    ) -> Self {
        let cancel_token = CancellationToken::new();
        let (instruction_tx, instruction_rx) = tokio::sync::mpsc::channel(10);

        Self {
            task,
            cancel_token,
            instruction_tx,
            instruction_rx: Some(instruction_rx),
            repo_pool,
            event_tx,
            worker,
            acp_config,
        }
    }
}

impl Actor for ExecutorActor {
    type Stop = ();

    async fn started(&mut self, _mailbox: &Mailbox<Self>) -> Result<(), Self::Stop> {
        let task = self.task.clone();
        let cancel_token = self.cancel_token.clone();
        let repo_pool = self.repo_pool.clone();
        let event_tx = self.event_tx.clone();
        let instruction_rx = self.instruction_rx.take().unwrap();
        let worker = self.worker.clone();
        let acp_config = self.acp_config.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.block_on(async {
                let result = execute_task(
                    task.clone(),
                    cancel_token,
                    instruction_rx,
                    event_tx,
                    repo_pool,
                    acp_config,
                ).await;

                if let Err(ref e) = result {
                    tracing::error!(task_id = %task.id, error = %e, "Task failed");
                }

                let _ = worker.send(TaskCompleted {
                    task_id: task.id,
                    result,
                }).await;
            });
        });

        Ok(())
    }

    async fn stopped(self) -> Self::Stop {}
}

async fn execute_task(
    task: TaskAssignment,
    cancel_token: CancellationToken,
    mut instruction_rx: tokio::sync::mpsc::Receiver<TaskInstruction>,
    event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
    repo_pool: Address<RepoPoolActor>,
    acp_config: AcpConfig,
) -> Result<()> {
    let slot_path = repo_pool
        .send(AcquireSlot {
            repo_url: task.repo_url.clone(),
            task_id: task.id,
            base_branch: task.base_branch.clone(),
            target_branch: task.target_branch.clone(),
            ssh_key: task.ssh_key.clone(),
        })
        .await
        .context("Failed to acquire slot")?
        .context("Failed to acquire repo slot")?;

    let workdir = std::fs::canonicalize(&slot_path)
        .context("Failed to resolve absolute path for workdir")?;

    if cancel_token.is_cancelled() {
        tracing::info!(task_id = %task.id, "Task cancelled before execution");
        let _ = repo_pool.send(ReleaseSlot {
            repo_url: task.repo_url.clone(),
            task_id: task.id,
        }).await;
        return Err(anyhow::anyhow!("Task cancelled before execution"));
    }

    let result = execute_agent(
        &task,
        &workdir,
        cancel_token.clone(),
        &mut instruction_rx,
        event_tx.clone(),
        &acp_config,
    ).await;

    if !cancel_token.is_cancelled() && result.is_ok() {
        tracing::info!(
            task_id = %task.id,
            target_branch = %task.target_branch,
            "Committing and pushing changes"
        );

        let git = GitOps::open(&slot_path)?;
        git.add_all()?;
        git.commit(&format!("Implement: {}", task.description))?;
        git.push(&task.target_branch, &task.ssh_key)?;

        tracing::info!(
            task_id = %task.id,
            target_branch = %task.target_branch,
            "Changes pushed successfully"
        );
    }

    if let Err(e) = repo_pool.send(ReleaseSlot {
        repo_url: task.repo_url.clone(),
        task_id: task.id,
    }).await {
        tracing::warn!(task_id = %task.id, error = %e, "Failed to release repo slot");
    }

    result
}

async fn execute_agent(
    task: &TaskAssignment,
    workdir: &PathBuf,
    cancel_token: CancellationToken,
    instruction_rx: &mut tokio::sync::mpsc::Receiver<TaskInstruction>,
    event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
    acp_config: &AcpConfig,
) -> Result<()> {
    let agent_config = acp_config
        .agent_servers
        .iter()
        .next()
        .map(|(_, v)| v.clone())
        .ok_or_else(|| anyhow::anyhow!("No agent servers configured in acp_config.json"))?;

    let mut cmd = tokio::process::Command::new(&agent_config.command);
    cmd.args(&agent_config.args)
        .current_dir(workdir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true);

    for (key, value) in &agent_config.env {
        cmd.env(key, value);
    }

    let mut child = cmd.spawn().context("Failed to spawn agent process")?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    let client = Arc::new(ACPClient::new(workdir.clone()));

    let cancel_token_clone = cancel_token.clone();
    let child = Arc::new(tokio::sync::Mutex::new(child));
    let child_clone = child.clone();

    tokio::spawn(async move {
        cancel_token_clone.cancelled().await;
        let mut child = child_clone.lock().await;
        #[cfg(unix)]
        {
            if let Some(pid) = child.id() {
                use nix::sys::signal::{Signal, kill};
                use nix::unistd::Pid;
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
            }
        }
        #[cfg(windows)]
        {
            let _ = child.kill().await;
        }
    });

    let local_set = tokio::task::LocalSet::new();
    let task_description = task.description.clone();
    let task_id = task.id;

    local_set
        .run_until(async move {
            let (conn, handle_io) =
                agent_client_protocol::ClientSideConnection::new(client.clone(), stdin.compat_write(), stdout.compat(), |fut| {
                    tokio::task::spawn_local(fut);
                });

            tokio::task::spawn_local(handle_io);

            conn.initialize(
                agent_client_protocol::InitializeRequest::new(
                    agent_client_protocol::ProtocolVersion::LATEST,
                )
                .client_capabilities(
                    AcpClientCapabilities::default()
                        .fs(FileSystemCapabilities::default().read_text_file(true).write_text_file(true))
                        .terminal(true),
                )
                .client_info(agent_client_protocol::Implementation::new("parallel-worker", "0.1.0")),
            )
            .await
            .context("Failed to initialize agent")?;

            let session = conn
                .new_session(agent_client_protocol::NewSessionRequest::new(workdir.clone()))
                .await
                .context("Failed to create session")?;

            tracing::info!("Agent session created: {:?}", session.session_id);

            let mut current_prompt = task_description.clone();

            loop {
                if cancel_token.is_cancelled() {
                    return Err(anyhow::anyhow!("Task cancelled"));
                }

                client.clear_messages().await;

                let text_content = agent_client_protocol::TextContent::new(&current_prompt);
                let content_block = ContentBlock::Text(text_content);

                let result = conn
                    .prompt(agent_client_protocol::PromptRequest::new(
                        session.session_id.clone(),
                        vec![content_block],
                    ))
                    .await;

                match result {
                    Ok(response) => match response.stop_reason {
                        acp::StopReason::EndTurn => {
                            let messages = client.get_messages().await;
                            let git = GitOps::open(workdir)?;
                            let diff = git.diff().unwrap_or_default();

                            tracing::info!("Task {} awaiting review", task_id);
                            let _ = event_tx.send(WorkerEvent::TaskAwaitingReview {
                                task_id,
                                messages,
                                diff,
                            }).await;

                            tokio::select! {
                                Some(instruction) = instruction_rx.recv() => {
                                    match instruction {
                                        TaskInstruction::Approve => {
                                            tracing::info!("Task {} approved, finalizing", task_id);
                                            return Ok(());
                                        }
                                        TaskInstruction::Iterate { feedback } => {
                                            tracing::info!("Task {} iterating with feedback", task_id);
                                            let _ = event_tx.send(WorkerEvent::TaskStarted {
                                                task_id,
                                            }).await;
                                            current_prompt = format!(
                                                "Human feedback: {}\n\nPlease improve the implementation based on this feedback.",
                                                feedback.message
                                            );
                                            continue;
                                        }
                                        TaskInstruction::Abort { reason } => {
                                            tracing::info!("Task {} aborted: {}", task_id, reason);
                                            return Err(anyhow::anyhow!("Task aborted: {}", reason));
                                        }
                                    }
                                }
                                _ = cancel_token.cancelled() => {
                                    return Err(anyhow::anyhow!("Task cancelled"));
                                }
                            }
                        }
                        acp::StopReason::MaxTokens => {
                            tracing::warn!("Agent hit max tokens limit");
                            current_prompt = "Please continue from where you left off.".to_string();
                            continue;
                        }
                        acp::StopReason::Refusal => {
                            tracing::warn!("Agent refused to continue");
                            return Err(anyhow::anyhow!("Agent refused to continue"));
                        }
                        acp::StopReason::Cancelled => {
                            return Err(anyhow::anyhow!("Task cancelled by client"));
                        }
                        acp::StopReason::MaxTurnRequests => {
                            tracing::warn!("Agent reached max turn requests");
                            current_prompt = "Please continue from where you left off.".to_string();
                            continue;
                        }
                        _ => {
                            tracing::warn!("Agent stopped with unhandled reason");
                            return Err(anyhow::anyhow!("Agent stopped with unhandled reason"));
                        }
                    },
                    Err(e) => {
                        tracing::error!("Prompt failed: {}", e);
                        return Err(anyhow::anyhow!("Prompt failed: {}", e));
                    }
                }
            }
        })
        .await
}

pub struct Cancel;
pub struct SendTaskInstruction(pub TaskInstruction);

impl xtra::Handler<Cancel> for ExecutorActor {
    type Return = ();

    async fn handle(&mut self, _msg: Cancel, _ctx: &mut xtra::Context<Self>) {
        self.cancel_token.cancel();
    }
}

impl xtra::Handler<SendTaskInstruction> for ExecutorActor {
    type Return = ();

    async fn handle(&mut self, msg: SendTaskInstruction, _ctx: &mut xtra::Context<Self>) {
        let _ = self.instruction_tx.send(msg.0).await;
    }
}
