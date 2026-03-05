use super::{IterationResult, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStage {
    Cloning,
    Working,
    Committing,
    Pushing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressUpdate {
    pub stage: ExecutionStage,
    pub message: String,
    pub percentage: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerMessage {
    TaskProgress {
        task_id: Uuid,
        progress: TaskProgressUpdate,
    },
    AgentOutput {
        task_id: Uuid,
        output: String,
    },
    TerminalOutput {
        task_id: Uuid,
        terminal_id: Uuid,
        output: String,
    },
    TaskCompleted {
        task_id: Uuid,
        result: IterationResult,
    },
    TaskStatusUpdate {
        task_id: Uuid,
        status: TaskStatus,
    },
    Heartbeat {
        worker_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    TaskCancelled {
        task_id: Uuid,
    },
    HumanAttached {
        task_id: Uuid,
        session_id: Uuid,
    },
    HumanDetached {
        task_id: Uuid,
        session_id: Uuid,
    },
    HumanMessage {
        task_id: Uuid,
        session_id: Uuid,
        message: String,
    },
    TerminalCommand {
        task_id: Uuid,
        terminal_id: Uuid,
        command: String,
    },
    AbortTask {
        task_id: Uuid,
        reason: String,
    },
    AcceptWork {
        task_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HumanMessage {
    SendMessage {
        task_id: Uuid,
        message: String,
    },
    TerminalInput {
        task_id: Uuid,
        terminal_id: Uuid,
        input: String,
    },
    AbortTask {
        task_id: Uuid,
    },
    AcceptWork {
        task_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HumanNotification {
    TaskProgress {
        task_id: Uuid,
        update: TaskProgressUpdate,
    },
    AgentOutput {
        task_id: Uuid,
        output: String,
    },
    TerminalOutput {
        task_id: Uuid,
        terminal_id: Uuid,
        output: String,
    },
    TaskAwaitingReview {
        task_id: Uuid,
        result: IterationResult,
    },
    TaskCompleted {
        task_id: Uuid,
        branch: String,
    },
    TaskStatusUpdate {
        task_id: Uuid,
        status: TaskStatus,
    },
}
