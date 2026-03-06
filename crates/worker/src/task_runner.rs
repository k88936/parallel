use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use agent_client_protocol as acp;
use agent_client_protocol::{Agent as _, ClientCapabilities as AcpClientCapabilities, FileSystemCapability, ContentBlock};

use parallel_protocol::{HumanFeedback, WorkerEvent};

use crate::acp_client::ACPClient;
use crate::repo_ops::GitOps;

pub enum TaskInstruction {
    Approve,
    Iterate { feedback: HumanFeedback },
    Abort { reason: String },
}

pub struct TaskRunner {
    task_id: uuid::Uuid,
    task_description: String,
    workdir: PathBuf,
}

impl TaskRunner {
    pub fn new(
        task_id: uuid::Uuid,
        task_description: String,
        workdir: PathBuf,
    ) -> Self {
        Self {
            task_id,
            task_description,
            workdir,
        }
    }

   pub async fn run(
       self,
       cancel_token: CancellationToken,
       event_tx: mpsc::Sender<WorkerEvent>,
       mut instruction_rx: mpsc::Receiver<TaskInstruction>,
    ) -> Result<()> {
        let agent_path = which::which("opencode")
            .map_err(|_| anyhow::anyhow!("Agent binary not found"))?;

        let mut child = tokio::process::Command::new(&agent_path)
            .args(["acp"])
            .current_dir(&self.workdir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn agent process")?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let client = Arc::new(ACPClient::new(self.workdir.clone()));

        let cancel_token_clone = cancel_token.clone();
        let pid = child.id();

        tokio::spawn(async move {
            cancel_token_clone.cancelled().await;
            if let Some(pid) = pid {
                #[cfg(unix)]
                {
                    use nix::sys::signal::{kill, Signal};
                    use nix::unistd::Pid;
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
            }
        });

        let local_set = tokio::task::LocalSet::new();
        local_set
            .run_until(async {
                let (conn, handle_io) =
                    agent_client_protocol::ClientSideConnection::new(client.clone(), stdin.compat_write(), stdout.compat(), |fut| {
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

                // init acp session
                let session = conn
                    .new_session(
                        agent_client_protocol::NewSessionRequest::new(self.workdir.clone())
                    )
                    .await
                    .context("Failed to create session")?;

                tracing::info!("Agent session created: {:?}", session.session_id);

                let mut current_prompt = self.task_description.clone();

                // iteration loop
                loop {
                    if cancel_token.is_cancelled() {
                        return Err(anyhow::anyhow!("Task cancelled"));
                    }

                    client.clear_messages().await;

                    let text_content = agent_client_protocol::TextContent::new(&current_prompt);
                    let content_block = ContentBlock::Text(text_content);

                    let result = conn
                        .prompt(
                            agent_client_protocol::PromptRequest::new(session.session_id.clone(), vec![content_block])
                        )
                        .await;

                    match result {
                        Ok(response) => {
                            match response.stop_reason {
                                acp::StopReason::EndTurn => {
                                    let messages = client.get_messages().await;
                                    let git = GitOps::open(&self.workdir)?;
                                    let diff = git.diff().unwrap_or_default();

                                    tracing::info!("Task {} awaiting review", self.task_id);
                                    event_tx.send(WorkerEvent::TaskAwaitingReview {
                                        task_id: self.task_id,
                                        messages,
                                        diff,
                                    }).await?;

                                    // wait for human in the loop
                                    tokio::select! {
                                        Some(instruction) = instruction_rx.recv() => {
                                            match instruction {
                                                TaskInstruction::Approve => {
                                                    tracing::info!("Task {} approved, finalizing", self.task_id);
                                                    return Ok(());
                                                }
                                                TaskInstruction::Iterate { feedback } => {
                                                    tracing::info!("Task {} iterating with feedback", self.task_id);
                                                    event_tx.send(WorkerEvent::TaskStarted {
                                                        task_id: self.task_id,
                                                    }).await?;
                                                    current_prompt = format!(
                                                        "Human feedback: {}\n\nPlease improve the implementation based on this feedback.",
                                                        feedback.message
                                                    );
                                                    continue;
                                                }
                                                TaskInstruction::Abort { reason } => {
                                                    tracing::info!("Task {} aborted: {}", self.task_id, reason);
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
                            }
                        }
                        Err(e) => {
                            tracing::error!("Prompt failed: {}", e);
                            return Err(anyhow::anyhow!("Prompt failed: {}", e));
                        }
                    }
                }
            })
            .await
    }
}
