use crate::ResourceMonitor;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum IterationStatus {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FeedbackType {
    Approve,
    RequestChanges,
    Abort,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HumanFeedback {
    #[ts(as = "String")]
    pub provided_at: DateTime<Utc>,
    pub feedback_type: FeedbackType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AgentMessage {
    #[ts(as = "String")]
    pub timestamp: DateTime<Utc>,
    pub role: String,
    pub message_type: MessageType,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewData {
    pub messages: Vec<AgentMessage>,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskAssignment {
    #[ts(type = "string")]
    pub id: Uuid,
    pub repo_url: String,
    pub description: String,
    pub base_branch: String,
    pub target_branch: String,
    pub ssh_key: String,
    pub max_execution_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum WorkerInstruction {
    AssignTask {
        task: TaskAssignment,
    },
    CancelTask {
        #[ts(type = "string")]
        task_id: Uuid,
        reason: String,
    },
    UpdateTask {
        #[ts(type = "string")]
        task_id: Uuid,
        instruction: String,
    },
    ApproveIteration {
        #[ts(type = "string")]
        task_id: Uuid,
    },
    ProvideFeedback {
        #[ts(type = "string")]
        task_id: Uuid,
        feedback: HumanFeedback,
    },
    AbortTask {
        #[ts(type = "string")]
        task_id: Uuid,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum WorkerEvent {
    Heartbeat {
        #[ts(type = "string[]")]
        running_tasks: Vec<Uuid>,
    },
    ResourceMonitor {
        resources: ResourceMonitor,
    },
    TaskStarted {
        #[ts(type = "string")]
        task_id: Uuid,
    },
    TaskProgress {
        #[ts(type = "string")]
        task_id: Uuid,
        message: String,
    },
    TaskAwaitingReview {
        #[ts(type = "string")]
        task_id: Uuid,
        messages: Vec<AgentMessage>,
        diff: String,
    },
    TaskCompleted {
        #[ts(type = "string")]
        task_id: Uuid,
    },
    TaskFailed {
        #[ts(type = "string")]
        task_id: Uuid,
        error: String,
    },
    TaskCancelled {
        #[ts(type = "string")]
        task_id: Uuid,
    },
}
