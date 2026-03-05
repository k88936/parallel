use anyhow::Result;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

use crate::protocol::{Task, TaskStatus, TaskPriority, TaskIteration};
use crate::server::db::entity::{tasks, task_iterations};

pub struct TaskScheduler {
    db: DatabaseConnection,
}

impl TaskScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_task(
        &self,
        repo_url: String,
        description: String,
        base_branch: String,
        target_branch: String,
        priority: TaskPriority,
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
            current_iteration: Set(0),
        };

        tasks::Entity::insert(task)
            .exec(&self.db)
            .await?;

        Ok(task_id)
    }

    pub async fn claim_task(&self, worker_id: &Uuid) -> Result<Option<Task>> {
        let now = Utc::now();

        let txn = self.db.begin().await?;

        let task = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Queued.as_str()))
            .order_by_desc(tasks::Column::Priority)
            .order_by_asc(tasks::Column::CreatedAt)
            .one(&txn)
            .await?;

        if let Some(task_model) = task {
            let task_id = task_model.id;
            let new_iteration = task_model.current_iteration + 1;

            let mut task_update: tasks::ActiveModel = task_model.into();
            task_update.status = Set(TaskStatus::Claimed.as_str().to_string());
            task_update.claimed_by = Set(Some(*worker_id));
            task_update.updated_at = Set(now);
            task_update.current_iteration = Set(new_iteration);
            task_update.update(&txn).await?;

            let iteration = task_iterations::ActiveModel {
                id: NotSet,
                task_id: Set(task_id),
                iteration_id: Set(new_iteration),
                started_at: Set(now),
                completed_at: Set(None),
                result_json: Set(None),
                human_feedback_json: Set(None),
            };

            task_iterations::Entity::insert(iteration)
                .exec(&txn)
                .await?;

            txn.commit().await?;

            let full_task = self.get_task(&task_id).await?;
            Ok(Some(full_task))
        } else {
            txn.commit().await?;
            Ok(None)
        }
    }

    pub async fn get_task(&self, task_id: &Uuid) -> Result<Task> {
        let task_model = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let iterations = task_iterations::Entity::find()
            .filter(task_iterations::Column::TaskId.eq(*task_id))
            .order_by_asc(task_iterations::Column::IterationId)
            .all(&self.db)
            .await?;

        let iterations = iterations
            .into_iter()
            .map(|iter| {
                let result = iter.result_json
                    .map(|json| serde_json::from_str(&json).unwrap());

                let human_feedback = iter.human_feedback_json
                    .map(|json| serde_json::from_str(&json).unwrap());

                TaskIteration {
                    iteration_id: iter.iteration_id as u32,
                    started_at: iter.started_at,
                    completed_at: iter.completed_at,
                    result,
                    human_feedback,
                }
            })
            .collect();

        Ok(Task {
            id: task_model.id,
            repo_url: task_model.repo_url,
            description: task_model.description,
            base_branch: task_model.base_branch,
            target_branch: task_model.target_branch,
            status: TaskStatus::from_str(&task_model.status)
                .unwrap_or(TaskStatus::Created),
            priority: TaskPriority::from_i32(task_model.priority)
                .unwrap_or(TaskPriority::Normal),
            created_at: task_model.created_at,
            updated_at: task_model.updated_at,
            claimed_by: task_model.claimed_by,
            iterations,
            current_iteration: task_model.current_iteration as u32,
        })
    }

    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<Task>> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        query = query
            .order_by_desc(tasks::Column::CreatedAt)
            .limit(limit)
            .offset(offset);

        let tasks = query.all(&self.db).await?;

        let mut result = Vec::new();
        for task_model in tasks {
            let task = self.get_task(&task_model.id).await?;
            result.push(task);
        }

        Ok(result)
    }

    pub async fn update_task_status(&self, task_id: &Uuid, status: TaskStatus) -> Result<()> {
        let now = Utc::now();

        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(status.as_str().to_string());
        task.updated_at = Set(now);
        task.update(&self.db).await?;

        Ok(())
    }

    pub async fn cancel_task(&self, task_id: &Uuid) -> Result<()> {
        self.update_task_status(task_id, TaskStatus::Cancelled).await
    }

    pub async fn count_tasks(&self, status: Option<TaskStatus>) -> Result<u64> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        let count = query.count(&self.db).await?;

        Ok(count)
    }
}