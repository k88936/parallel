use crate::protocol::{HumanFeedback, Task as ProtocolTask, WorkerCapabilities, WorkerInstruction, WorkerEvent};
use crate::worker::git::GitOps;
use crate::worker::api_client::APIClient;
use crate::worker::task::Task;
use agent_client_protocol as acp;
use agent_client_protocol::{Agent as _, ClientCapabilities as AcpClientCapabilities, FileSystemCapability, ContentBlock};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tracing::{error, info, warn};
use uuid::Uuid;

enum TaskInstruction {
    Approve,
    Iterate { feedback: HumanFeedback },
    Abort { reason: String },
}

struct RunningTask {
    cancel_token: CancellationToken,
    instruction_tx: mpsc::Sender<TaskInstruction>,
}

pub struct Worker {
    work_base: PathBuf,
    agent_path: PathBuf,
    max_concurrent: usize,
    api_client: Arc<APIClient>,
    worker_id: Uuid,
    ssh_key_path: PathBuf,
    running_tasks: Arc<RwLock<HashMap<Uuid, RunningTask>>>,
    event_tx: mpsc::Sender<WorkerEvent>,
    event_rx: Option<mpsc::Receiver<WorkerEvent>>,
}

impl Worker {
    pub fn new(
        work_base: PathBuf,
        agent_path: PathBuf,
        max_concurrent: usize,
        server_url: String,
        ssh_key_path: PathBuf,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);
        
        Self {
            work_base,
            agent_path,
            max_concurrent,
            api_client: Arc::new(APIClient::new(server_url)),
            worker_id: Uuid::nil(),
            ssh_key_path,
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Some(event_rx),
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
        if self.worker_id == Uuid::nil() {
            anyhow::bail!("Worker not registered. Call register() first.");
        }

        info!("Worker {} starting main loop", self.worker_id);

        let event_rx = self.event_rx.take().unwrap();
        let worker_id = self.worker_id;
        let api_client = self.api_client.clone();
        let running_tasks = self.running_tasks.clone();

        tokio::spawn(async move {
            Self::event_sender_loop(api_client, worker_id, event_rx, running_tasks).await;
        });

        let mut poll_interval = tokio::time::interval(Duration::from_secs(2));

        loop {
            tokio::select! {
                _ = poll_interval.tick() => {
                    match self.api_client.poll_instructions(self.worker_id).await {
                        Ok(instructions) => {
                            for instruction in instructions {
                                self.handle_instruction(instruction).await;
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

    async fn event_sender_loop(
        api_client: Arc<APIClient>,
        worker_id: Uuid,
        mut event_rx: mpsc::Receiver<WorkerEvent>,
        running_tasks: Arc<RwLock<HashMap<Uuid, RunningTask>>>,
    ) {
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(10));
        let mut pending_events: Vec<WorkerEvent> = Vec::new();

        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    let tasks = running_tasks.read().await;
                    let running_task_ids: Vec<Uuid> = tasks.keys().copied().collect();
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
                
                match api_client.push_events(worker_id, events_to_send.clone()).await {
                    Ok(true) => {
                        pending_events.clear();
                    }
                    Ok(false) | Err(_) => {
                        pending_events = events_to_send;
                    }
                }
            }
        }
    }

    async fn handle_instruction(&self, instruction: WorkerInstruction) {
        match instruction {
            WorkerInstruction::AssignTask { task } => {
                let running = self.running_tasks.read().await;
                if running.len() >= self.max_concurrent {
                    warn!("Max concurrent tasks reached, cannot accept task {}", task.id);
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

                let worker_task = self.protocol_to_worker_task(&task);
                let event_tx = self.event_tx.clone();
                let running_tasks = self.running_tasks.clone();
                let work_base = self.work_base.clone();
                let agent_path = self.agent_path.clone();
                let ssh_key_path = self.ssh_key_path.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create tokio runtime");
                    
                    rt.block_on(async {
                        let _ = event_tx.send(WorkerEvent::TaskStarted { task_id }).await;
                        
                        let result = Self::execute_task(
                            &worker_task,
                            &work_base,
                            &agent_path,
                            &ssh_key_path,
                            cancel_token,
                            instruction_rx,
                            event_tx.clone(),
                        ).await;

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
                                let _ = event_tx.send(WorkerEvent::TaskFailed { 
                                    task_id, 
                                    error: e.to_string() 
                                }).await;
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
            WorkerInstruction::UpdateTask { task_id, instruction } => {
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
                    let _ = task.instruction_tx.send(TaskInstruction::Iterate { feedback }).await;
                } else {
                    warn!("Task {} not found for feedback", task_id);
                }
            }
            WorkerInstruction::AbortTask { task_id, reason } => {
                info!("Received abort for task {}: {}", task_id, reason);
                let running = self.running_tasks.read().await;
                if let Some(task) = running.get(&task_id) {
                    let _ = task.instruction_tx.send(TaskInstruction::Abort { reason }).await;
                } else {
                    warn!("Task {} not found for abort", task_id);
                }
            }
        }
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

    async fn execute_task(
        task: &Task,
        work_base: &PathBuf,
        agent_path: &PathBuf,
        ssh_key_path: &PathBuf,
        cancel_token: CancellationToken,
        mut instruction_rx: mpsc::Receiver<TaskInstruction>,
        event_tx: mpsc::Sender<WorkerEvent>,
    ) -> Result<()> {
        let task_dir = work_base.join(&task.id);
        tokio::fs::create_dir_all(&task_dir)
            .await
            .context("Failed to create task directory")?;

        let repo_dir = task_dir.join("repo");
        let git = GitOps::clone(&task.repo_url, &repo_dir, ssh_key_path)?;
        
        if cancel_token.is_cancelled() {
            return Err(anyhow::anyhow!("Task cancelled before execution"));
        }
        
        git.create_branch(&task.branch_name)?;

        let workdir = std::fs::canonicalize(&repo_dir)
            .context("Failed to resolve absolute path for workdir")?;
        
        let agent_path = which::which(agent_path)
            .map_err(|_| anyhow::anyhow!("Agent binary not found: {:?}", agent_path))?;

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

        let client = Arc::new(crate::worker::acp_client::ACPClient::new(workdir.clone()));

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

                let session = conn
                    .new_session(
                        agent_client_protocol::NewSessionRequest::new(workdir.clone())
                    )
                    .await
                    .context("Failed to create session")?;

                info!("Agent session created: {:?}", session.session_id);

                let mut current_prompt = task.description.clone();
                let task_id = uuid::Uuid::parse_str(&task.id).unwrap_or_else(|_| uuid::Uuid::nil());

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
                            info!("Prompt completed with stop reason: {:?}", response.stop_reason);
                            
                            match response.stop_reason {
                                // should review
                                acp::StopReason::EndTurn => {
                                    let messages = client.get_messages().await;
                                    let diff = git.diff().unwrap_or_default();
                                    
                                    info!("Task {} awaiting review", task_id);
                                    let _ = event_tx.send(WorkerEvent::TaskAwaitingReview {
                                        task_id,
                                        messages,
                                        diff,
                                    }).await;

                                    tokio::select! {
                                        Some(instruction) = instruction_rx.recv() => {
                                            match instruction {
                                                TaskInstruction::Approve => {
                                                    info!("Task {} approved, finalizing", task_id);
                                                    
                                                    git.add_all()?;
                                                    git.commit(&format!("Implement: {}", task.description))?;
                                                    git.push(&task.branch_name, ssh_key_path)?;
                                                    
                                                    if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
                                                        warn!("Failed to cleanup task dir: {}", e);
                                                    }
                                                    return Ok(());
                                                }
                                                TaskInstruction::Iterate { feedback } => {
                                                    info!("Task {} iterating with feedback", task_id);
                                                    current_prompt = format!(
                                                        "Human feedback: {}\n\nPlease improve the implementation based on this feedback.",
                                                        feedback.message
                                                    );
                                                    continue;
                                                }
                                                TaskInstruction::Abort { reason } => {
                                                    info!("Task {} aborted: {}", task_id, reason);
                                                    if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
                                                        warn!("Failed to cleanup task dir: {}", e);
                                                    }
                                                    return Err(anyhow::anyhow!("Task aborted: {}", reason));
                                                }
                                            }
                                        }
                                        _ = cancel_token.cancelled() => {
                                            if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
                                                warn!("Failed to cleanup cancelled task dir: {}", e);
                                            }
                                            return Err(anyhow::anyhow!("Task cancelled"));
                                        }
                                    }
                                }
                                acp::StopReason::MaxTokens => {
                                    warn!("Agent hit max tokens limit");
                                    current_prompt = "Please continue from where you left off.".to_string();
                                    continue;
                                }
                                acp::StopReason::Refusal => {
                                    warn!("Agent refused to continue");
                                    if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
                                        warn!("Failed to cleanup task dir: {}", e);
                                    }
                                    return Err(anyhow::anyhow!("Agent refused to continue"));
                                }
                                acp::StopReason::Cancelled => {
                                    if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
                                        warn!("Failed to cleanup cancelled task dir: {}", e);
                                    }
                                    return Err(anyhow::anyhow!("Task cancelled by client"));
                                }
                                acp::StopReason::MaxTurnRequests => {
                                    warn!("Agent reached max turn requests");
                                    current_prompt = "Please continue from where you left off.".to_string();
                                    continue;
                                }
                                _ => {
                                    warn!("Agent stopped with unhandled reason");
                                    if let Err(e) = tokio::fs::remove_dir_all(&task_dir).await {
                                        warn!("Failed to cleanup task dir: {}", e);
                                    }
                                    return Err(anyhow::anyhow!("Agent stopped with unhandled reason"));
                                }
                            }
                        }
                        Err(e) => {
                            error!("Prompt failed: {}", e);
                            return Err(anyhow::anyhow!("Prompt failed: {}", e));
                        }
                    }
                }
            })
            .await
    }
}
