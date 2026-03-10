use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use xtra::{Actor, Address, Mailbox};

use parallel_common::{WorkerEvent, WorkerInstruction};

use crate::Config;
use crate::actors::{Cancel, ExecutorActor, RepoPoolActor, SendTaskInstruction, TaskInstruction};

pub struct HandleInstruction(pub WorkerInstruction);

pub struct TaskCompleted {
    pub task_id: Uuid,
    pub result: Result<()>,
}

struct RunningTask {
    task_address: Address<ExecutorActor>,
}

pub struct ManagerActor {
    config: Config,
    repo_pool_addr: Address<RepoPoolActor>,
    self_addr: Address<Self>,
    event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
    running_tasks: HashMap<Uuid, RunningTask>,
}
impl ManagerActor {
    pub(crate) fn new(
        config: Config,
        repo_pool_addr: Address<RepoPoolActor>,
        self_addr: Address<Self>,
        event_tx: tokio::sync::mpsc::Sender<WorkerEvent>,
    ) -> Self {
        Self {
            config,
            repo_pool_addr,
            self_addr,
            event_tx,
            running_tasks: Default::default(),
        }
    }
}

impl Actor for ManagerActor {
    type Stop = ();
    async fn stopped(self) -> Self::Stop {}
}
impl xtra::Handler<HandleInstruction> for ManagerActor {
    type Return = ();

    async fn handle(&mut self, msg: HandleInstruction, _ctx: &mut xtra::Context<Self>) {
        match msg.0 {
            WorkerInstruction::AssignTask { task } => {
                if self.running_tasks.len() >= self.config.max_concurrent {
                    tracing::warn!(
                        task_id = %task.id,
                        running_count = self.running_tasks.len(),
                        max_concurrent = self.config.max_concurrent,
                        "Max concurrent tasks reached"
                    );
                    return;
                }

                let task_id = task.id;
                tracing::info!(
                    task_id = %task_id,
                    repo_url = %task.repo_url,
                    "Received task assignment"
                );

                let repo_pool = self.repo_pool_addr.clone();
                let event_tx = self.event_tx.clone();
                let worker_addr = self.self_addr.clone();

                let (task_addr, task_mailbox) = Mailbox::unbounded();
                let task_actor = ExecutorActor::new(task, repo_pool, event_tx, worker_addr);
                xtra::spawn_tokio(task_actor, (task_addr.clone(), task_mailbox));

                self.running_tasks.insert(
                    task_id,
                    RunningTask {
                        task_address: task_addr.clone(),
                    },
                );

                let _ = self
                    .event_tx
                    .send(WorkerEvent::TaskStarted { task_id })
                    .await;
            }
            WorkerInstruction::CancelTask { task_id, reason } => {
                tracing::info!(
                    task_id = %task_id,
                    reason = %reason,
                    "Received cancel request"
                );

                if let Some(task) = self.running_tasks.get(&task_id) {
                    let _ = task.task_address.send(Cancel).await;
                }
            }
            WorkerInstruction::UpdateTask {
                task_id,
                instruction,
            } => {
                tracing::info!(
                    task_id = %task_id,
                    instruction = %instruction,
                    "Received update"
                );
            }
            WorkerInstruction::ApproveIteration { task_id } => {
                tracing::info!(
                    task_id = %task_id,
                    "Received approval"
                );
                if let Some(task) = self.running_tasks.get(&task_id) {
                    let _ = task
                        .task_address
                        .send(SendTaskInstruction(TaskInstruction::Approve))
                        .await;
                }
            }
            WorkerInstruction::ProvideFeedback { task_id, feedback } => {
                tracing::info!(
                    task_id = %task_id,
                    feedback_type = ?feedback.feedback_type,
                    "Received feedback"
                );
                if let Some(task) = self.running_tasks.get(&task_id) {
                    let _ = task
                        .task_address
                        .send(SendTaskInstruction(TaskInstruction::Iterate { feedback }))
                        .await;
                }
            }
            WorkerInstruction::AbortTask { task_id, reason } => {
                tracing::info!(
                    task_id = %task_id,
                    reason = %reason,
                    "Received abort"
                );
                if let Some(task) = self.running_tasks.get(&task_id) {
                    let _ = task
                        .task_address
                        .send(SendTaskInstruction(TaskInstruction::Abort { reason }))
                        .await;
                }
            }
        }
    }
}

impl xtra::Handler<TaskCompleted> for ManagerActor {
    type Return = ();

    async fn handle(&mut self, msg: TaskCompleted, _ctx: &mut xtra::Context<Self>) {
        self.running_tasks.remove(&msg.task_id);

        let event = match msg.result {
            Ok(()) => {
                tracing::info!(task_id = %msg.task_id, "Task completed");
                WorkerEvent::TaskCompleted {
                    task_id: msg.task_id,
                }
            }
            Err(e) => {
                tracing::error!(task_id = %msg.task_id, error = %e, "Task failed");
                WorkerEvent::TaskFailed {
                    task_id: msg.task_id,
                    error: e.to_string(),
                }
            }
        };
        let _ = self.event_tx.send(event).await;
    }
}

pub struct GetRunningTaskIds;

impl xtra::Handler<GetRunningTaskIds> for ManagerActor {
    type Return = Vec<Uuid>;

    async fn handle(
        &mut self,
        _msg: GetRunningTaskIds,
        _ctx: &mut xtra::Context<Self>,
    ) -> Self::Return {
        self.running_tasks.keys().copied().collect()
    }
}
