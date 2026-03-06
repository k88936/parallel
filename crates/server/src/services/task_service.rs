use anyhow::Result;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{ReviewData, Task, TaskPriority, TaskStatus};

use crate::db::entity::tasks;
use crate::errors::{ServerError, ServerResult};

pub struct TaskService {
    db: DatabaseConnection,
}

impl TaskService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        repo_url: String,
        description: String,
        base_branch: String,
        target_branch: String,
        priority: TaskPriority,
        ssh_key: String,
    ) -> Result<Uuid> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let task = tasks::ActiveModel {
            id: Set(task_id),
            repo_url: Set(repo_url),
            description: Set(description),
            base_branch: Set(base_branch),
            target_branch: Set(target_branch),
            status: Set(TaskStatus::Queued.as_str().to_string()),
            priority: Set(priority.as_i32()),
            created_at: Set(now),
            updated_at: Set(now),
            claimed_by: Set(None),
            review_data_json: Set(None),
            ssh_key: Set(ssh_key),
        };

        tasks::Entity::insert(task).exec(&self.db).await?;

        Ok(task_id)
    }

    pub async fn get(&self, task_id: &Uuid) -> ServerResult<Task> {
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        Ok(Task {
            id: task.id,
            repo_url: task.repo_url,
            description: task.description,
            base_branch: task.base_branch,
            target_branch: task.target_branch,
            status: TaskStatus::from_str(&task.status).unwrap_or(TaskStatus::Created),
            priority: TaskPriority::from_i32(task.priority).unwrap_or(TaskPriority::Normal),
            created_at: task.created_at,
            updated_at: task.updated_at,
            claimed_by: task.claimed_by,
            ssh_key: task.ssh_key,
        })
    }

    pub async fn list(
        &self,
        status: Option<TaskStatus>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<Task>> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        let tasks = query
            .order_by_desc(tasks::Column::CreatedAt)
            .limit(limit.unwrap_or(50))
            .offset(offset.unwrap_or(0))
            .all(&self.db)
            .await?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(Task {
                id: task.id,
                repo_url: task.repo_url,
                description: task.description,
                base_branch: task.base_branch,
                target_branch: task.target_branch,
                status: TaskStatus::from_str(&task.status).unwrap_or(TaskStatus::Created),
                priority: TaskPriority::from_i32(task.priority).unwrap_or(TaskPriority::Normal),
                created_at: task.created_at,
                updated_at: task.updated_at,
                claimed_by: task.claimed_by,
                ssh_key: task.ssh_key,
            });
        }

        Ok(result)
    }

    pub async fn count(&self, status: Option<TaskStatus>) -> Result<u64> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        Ok(query.count(&self.db).await?)
    }

    pub async fn update_status(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(status.as_str().to_string());
        task.updated_at = Set(now);
        task.update(&self.db).await?;

        Ok(())
    }

    pub async fn set_claimed_by(&self, task_id: &Uuid, worker_id: Option<Uuid>) -> ServerResult<()> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let mut task: tasks::ActiveModel = task.into();
        task.claimed_by = Set(worker_id);
        task.updated_at = Set(now);
        task.update(&self.db).await?;

        Ok(())
    }

    pub async fn complete_iteration(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(status.as_str().to_string());
        task.updated_at = Set(now);

        if status == TaskStatus::Completed || status == TaskStatus::Cancelled {
            task.claimed_by = Set(None);
        }

        task.update(&self.db).await?;
        Ok(())
    }

    pub async fn set_review_data(&self, task_id: &Uuid, review_data: ReviewData) -> ServerResult<()> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(TaskStatus::AwaitingReview.as_str().to_string());
        task.review_data_json = Set(Some(serde_json::to_string(&review_data)?));
        task.updated_at = Set(now);
        task.update(&self.db).await?;

        Ok(())
    }

    pub async fn get_review_data(&self, task_id: &Uuid) -> ServerResult<Option<ReviewData>> {
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        match task.review_data_json {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    pub async fn get_next_queued(&self) -> Result<Option<Task>> {
        let task = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Queued.as_str()))
            .order_by_desc(tasks::Column::Priority)
            .order_by_asc(tasks::Column::CreatedAt)
            .one(&self.db)
            .await?;

        Ok(task.map(|t| Task {
            id: t.id,
            repo_url: t.repo_url,
            description: t.description,
            base_branch: t.base_branch,
            target_branch: t.target_branch,
            status: TaskStatus::from_str(&t.status).unwrap_or(TaskStatus::Created),
            priority: TaskPriority::from_i32(t.priority).unwrap_or(TaskPriority::Normal),
            created_at: t.created_at,
            updated_at: t.updated_at,
            claimed_by: t.claimed_by,
            ssh_key: t.ssh_key,
        }))
    }
}
