use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use parallel_domain::{ReviewData, TaskAssignment, TaskDTO, TaskPriority, TaskStatus};

use crate::common::types::Task;
use crate::errors::{ServerError, ServerResult};
use crate::repository::{TaskRepository, TaskRepositoryTrait};

impl Task {
    pub fn to_dto(&self) -> TaskDTO {
        TaskDTO {
            id: self.id,
            title: self.title.clone(),
            repo_url: self.repo_url.clone(),
            description: self.description.clone(),
            base_branch: self.base_branch.clone(),
            target_branch: self.target_branch.clone(),
            status: self.status,
            priority: self.priority,
            created_at: self.created_at,
            updated_at: self.updated_at,
            claimed_by: self.claimed_by,
        }
    }

    pub fn to_assignment(&self) -> TaskAssignment {
        TaskAssignment {
            id: self.id,
            repo_url: self.repo_url.clone(),
            description: self.description.clone(),
            base_branch: self.base_branch.clone(),
            target_branch: self.target_branch.clone(),
            ssh_key: self.ssh_key.clone(),
        }
    }
}

pub struct TaskService {
    repository: Arc<TaskRepository>,
}

impl TaskService {
    pub fn new(repository: Arc<TaskRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl TaskServiceTrait for TaskService {
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
        project_id: Option<Uuid>,
    ) -> Result<Uuid> {
        let task_id = Uuid::new_v4();

        self.repository.create(
            task_id,
            title,
            repo_url,
            description,
            base_branch,
            target_branch,
            TaskStatus::Queued,
            priority,
            ssh_key,
            max_execution_time,
            project_id,
        ).await?;

        Ok(task_id)
    }

    async fn get(&self, task_id: &Uuid) -> ServerResult<TaskDTO> {
        let task = self.repository.find_by_id(task_id).await?;
        Ok(task.to_dto())
    }

    async fn get_entity(&self, task_id: &Uuid) -> ServerResult<Task> {
        self.repository.find_by_id(task_id).await
    }

    async fn list(&self, params: TaskListParams) -> Result<TaskListResult> {
        let limit = params.limit.unwrap_or(50);
        let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
        let sort_direction = params.sort_direction.as_deref().unwrap_or("desc");

        let (tasks, has_more) = self.repository.find_many(
            params.status,
            params.priority,
            params.repo_url.as_deref(),
            params.worker_id,
            params.search.as_deref(),
            params.created_after,
            params.created_before,
            params.project_id,
            sort_by,
            sort_direction,
            params.cursor.as_deref(),
            limit,
        ).await?;

        let total = self.repository.count(params.status).await?;

        let next_cursor = if has_more {
            if let Some(last) = tasks.last() {
                TaskRepository::encode_cursor_for_task(last, sort_by)
            } else {
                None
            }
        } else {
            None
        };

        let task_dtos: Vec<TaskDTO> = tasks.iter().map(|t| t.to_dto()).collect();

        Ok(TaskListResult {
            tasks: task_dtos,
            total,
            next_cursor,
            has_more,
        })
    }

    async fn count(&self, status: Option<TaskStatus>) -> Result<u64> {
        self.repository.count(status).await
    }

    async fn update_status(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
        self.repository.update_status(task_id, status).await
    }

    async fn set_claimed_by(&self, task_id: &Uuid, worker_id: Option<Uuid>) -> ServerResult<()> {
        self.repository.set_claimed_by(task_id, worker_id).await
    }

    async fn complete_iteration(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
        self.repository.complete_iteration(task_id, status).await
    }

    async fn set_review_data(&self, task_id: &Uuid, review_data: ReviewData) -> ServerResult<()> {
        self.repository.set_review_data(task_id, TaskStatus::AwaitingReview, &review_data).await
    }

    async fn get_review_data(&self, task_id: &Uuid) -> ServerResult<Option<ReviewData>> {
        self.repository.get_review_data(task_id).await
    }

    async fn get_next_queued(&self) -> Result<Option<Task>> {
        self.repository.find_next_queued().await
    }

    async fn requeue_task(&self, task_id: &Uuid) -> ServerResult<()> {
        let task = self.repository.find_by_id(task_id).await?;

        if !matches!(
            task.status,
            TaskStatus::InProgress | TaskStatus::Claimed | TaskStatus::AwaitingReview
        ) {
            return Ok(());
        }

        self.repository.requeue(task_id).await?;
        Ok(())
    }

    async fn requeue_tasks(&self, task_ids: &[Uuid]) -> ServerResult<usize> {
        let mut count = 0;
        for task_id in task_ids {
            match self.requeue_task(task_id).await {
                Ok(()) => count += 1,
                Err(_) => continue,
            }
        }
        Ok(count)
    }

    async fn find_orphaned_tasks(&self) -> ServerResult<Vec<Task>> {
        self.repository.find_orphaned().await
    }

    async fn find_timed_out_tasks(&self) -> ServerResult<Vec<Task>> {
        self.repository.find_timed_out().await
    }

    async fn fail_task(&self, task_id: &Uuid, _reason: &str) -> ServerResult<()> {
        self.repository.fail(task_id).await
    }

    async fn retry_task(&self, task_id: &Uuid, clear_review_data: bool) -> ServerResult<TaskDTO> {
        let task = self.repository.find_by_id(task_id).await?;

        if !matches!(
            task.status,
            TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::Completed
        ) {
            return Err(ServerError::InvalidStatus(format!(
                "Task with status '{}' cannot be retried. Only Failed, Cancelled, or Completed tasks can be retried.",
                task.status.as_str()
            )));
        }

        let updated = self.repository.requeue(task_id).await?;
        
        if clear_review_data {
            self.repository.set_review_data(task_id, updated.status, &ReviewData {
                messages: vec![],
                diff: String::new(),
            }).await?;
        }

        let task = self.repository.find_by_id(task_id).await?;
        Ok(task.to_dto())
    }
}

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
    pub project_id: Option<Uuid>,
}

pub struct TaskListResult {
    pub tasks: Vec<TaskDTO>,
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
        project_id: Option<Uuid>,
    ) -> Result<Uuid>;

    async fn get(&self, task_id: &Uuid) -> ServerResult<TaskDTO>;

    async fn get_entity(&self, task_id: &Uuid) -> ServerResult<Task>;

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

    async fn retry_task(&self, task_id: &Uuid, clear_review_data: bool) -> ServerResult<TaskDTO>;
}
