use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use parallel_protocol::{
    HumanFeedback, ReviewData, Task, TaskPriority, TaskStatus, WorkerCapabilities, WorkerEvent,
    WorkerInfo, WorkerInstruction, WorkerStatus,
};

use crate::errors::ServerResult;

pub struct TaskListParams {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub repo_url: Option<String>,
    pub worker_id: Option<Uuid>,
    pub search: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub struct TaskListResult {
    pub tasks: Vec<Task>,
    pub total: u64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[async_trait]
pub trait TaskServiceTrait: Send + Sync {
    async fn create(
        &self,
        title: String,
        repo_url: String,
        description: String,
        base_branch: String,
        target_branch: String,
        priority: TaskPriority,
        ssh_key: String,
        max_execution_time: i64,
    ) -> Result<Uuid>;

    async fn get(&self, task_id: &Uuid) -> ServerResult<Task>;

    async fn list(&self, params: TaskListParams) -> Result<TaskListResult>;

    async fn count(&self, status: Option<TaskStatus>) -> Result<u64>;

    async fn update_status(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()>;

    async fn set_claimed_by(&self, task_id: &Uuid, worker_id: Option<Uuid>) -> ServerResult<()>;

    async fn complete_iteration(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()>;

    async fn set_review_data(&self, task_id: &Uuid, review_data: ReviewData) -> ServerResult<()>;

    async fn get_review_data(&self, task_id: &Uuid) -> ServerResult<Option<ReviewData>>;

    async fn get_next_queued(&self) -> Result<Option<Task>>;

    async fn requeue_task(&self, task_id: &Uuid) -> ServerResult<()>;

    async fn requeue_tasks(&self, task_ids: &[Uuid]) -> ServerResult<usize>;

    async fn find_orphaned_tasks(&self) -> ServerResult<Vec<Task>>;

    async fn find_timed_out_tasks(&self) -> ServerResult<Vec<Task>>;

    async fn fail_task(&self, task_id: &Uuid, reason: &str) -> ServerResult<()>;
}

#[async_trait]
pub trait WorkerServiceTrait: Send + Sync {
    async fn register(
        &self,
        name: String,
        capabilities: WorkerCapabilities,
        max_concurrent: usize,
    ) -> ServerResult<WorkerInfo>;

    async fn get(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo>;

    async fn list(&self) -> ServerResult<Vec<WorkerInfo>>;

    async fn update_heartbeat(
        &self,
        worker_id: &Uuid,
        running_tasks: Vec<Uuid>,
    ) -> ServerResult<()>;

    async fn add_task(&self, worker_id: &Uuid, task_id: Uuid) -> ServerResult<()>;

    async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool>;

    async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>>;

    async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()>;

    async fn find_stale_workers(
        &self,
        timeout_seconds: i64,
    ) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>>;

    async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()>;
}

#[async_trait]
pub trait CoordinatorTrait: Send + Sync {
    async fn queue_instruction(
        &self,
        worker_id: Uuid,
        instruction: WorkerInstruction,
    ) -> ServerResult<()>;

    async fn get_pending_instructions(
        &self,
        worker_id: &Uuid,
    ) -> ServerResult<Vec<WorkerInstruction>>;

    async fn queue_feedback(
        &self,
        worker_id: Uuid,
        task_id: Uuid,
        feedback: HumanFeedback,
    ) -> ServerResult<()>;

    async fn queue_cancellation(
        &self,
        worker_id: Uuid,
        task_id: Uuid,
        reason: String,
    ) -> ServerResult<()>;
}

#[async_trait]
pub trait EventProcessorTrait: Send + Sync {
    async fn process_events(&self, worker_id: &Uuid, events: Vec<WorkerEvent>) -> ServerResult<()>;
}
