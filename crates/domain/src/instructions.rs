use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::TaskAssignment;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IterationStatus {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    Approve,
    RequestChanges,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFeedback {
    pub provided_at: DateTime<Utc>,
    pub feedback_type: FeedbackType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub timestamp: DateTime<Utc>,
    pub role: String,
    pub message_type: MessageType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    Thought,
    Text,
    ToolCall,
    ToolResult,
    Image,
    Resource,
    Plan,
    UserMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    pub messages: Vec<AgentMessage>,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerInstruction {
    AssignTask {
        task: TaskAssignment,
    },
    CancelTask {
        task_id: Uuid,
        reason: String,
    },
    UpdateTask {
        task_id: Uuid,
        instruction: String,
    },
    ApproveIteration {
        task_id: Uuid,
    },
    ProvideFeedback {
        task_id: Uuid,
        feedback: HumanFeedback,
    },
    AbortTask {
        task_id: Uuid,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerEvent {
    Heartbeat {
        running_tasks: Vec<Uuid>,
    },
    TaskStarted {
        task_id: Uuid,
    },
    TaskProgress {
        task_id: Uuid,
        message: String,
    },
    TaskAwaitingReview {
        task_id: Uuid,
        messages: Vec<AgentMessage>,
        diff: String,
    },
    TaskCompleted {
        task_id: Uuid,
    },
    TaskFailed {
        task_id: Uuid,
        error: String,
    },
    TaskCancelled {
        task_id: Uuid,
    },
}
