use anyhow::Result;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{ReviewData, Task, TaskPriority, TaskStatus};

use crate::db::entity::tasks;
use crate::errors::{ServerError, ServerResult};
use crate::services::traits::{TaskListParams, TaskListResult, TaskServiceTrait};

pub struct TaskService {
    db: DatabaseConnection,
}

impl TaskService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_task(t: tasks::Model) -> Task {
    Task {
        id: t.id,
        title: t.title,
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
        max_execution_time: t.max_execution_time,
    }
}

fn decode_cursor(cursor: &str) -> Option<(String, Uuid)> {
    let decoded = STANDARD.decode(cursor).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = decoded_str.split('|').collect();
    if parts.len() != 2 {
        return None;
    }
    let value = parts[0].to_string();
    let id = Uuid::parse_str(parts[1]).ok()?;
    Some((value, id))
}

fn encode_cursor(value: &str, id: Uuid) -> String {
    STANDARD.encode(format!("{}|{}", value, id))
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
        let now = Utc::now();

        let task = tasks::ActiveModel {
            id: Set(task_id),
            title: Set(title),
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
            max_execution_time: Set(max_execution_time),
            project_id: Set(project_id),
        };

        tasks::Entity::insert(task).exec(&self.db).await?;

        Ok(task_id)
    }

    async fn get(&self, task_id: &Uuid) -> ServerResult<Task> {
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        Ok(model_to_task(task))
    }

    async fn list(&self, params: TaskListParams) -> Result<TaskListResult> {
        let limit = params.limit.unwrap_or(50);
        let fetch_limit = limit + 1;

        let mut query = tasks::Entity::find();

        if let Some(s) = params.status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        if let Some(p) = params.priority {
            query = query.filter(tasks::Column::Priority.eq(p.as_i32()));
        }

        if let Some(ref repo) = params.repo_url {
            query = query.filter(tasks::Column::RepoUrl.eq(repo));
        }

        if let Some(worker_id) = params.worker_id {
            query = query.filter(tasks::Column::ClaimedBy.eq(worker_id));
        }

        if let Some(ref search) = params.search {
            let pattern = format!("%{}%", search);
            query = query.filter(
                Condition::any()
                    .add(tasks::Column::Title.like(&pattern))
                    .add(tasks::Column::Description.like(&pattern)),
            );
        }

        if let Some(after) = params.created_after {
            query = query.filter(tasks::Column::CreatedAt.gte(after));
        }

        if let Some(before) = params.created_before {
            query = query.filter(tasks::Column::CreatedAt.lte(before));
        }

        if let Some(project_id) = params.project_id {
            query = query.filter(tasks::Column::ProjectId.eq(project_id));
        }

        let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
        let sort_direction = params.sort_direction.as_deref().unwrap_or("desc");
        let is_desc = sort_direction == "desc";

        let (sort_column, cursor_value) = match sort_by {
            "created_at" => (tasks::Column::CreatedAt, true),
            "updated_at" => (tasks::Column::UpdatedAt, true),
            "priority" => (tasks::Column::Priority, true),
            "status" => (tasks::Column::Status, false),
            _ => (tasks::Column::CreatedAt, true),
        };

        if let Some(ref cursor) = params.cursor {
            if let Some((value, id)) = decode_cursor(cursor) {
                if cursor_value {
                    if let Ok(ts) = value.parse::<i64>() {
                        let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or(Utc::now());
                        if is_desc {
                            query = query.filter(
                                Condition::any()
                                    .add(sort_column.lt(dt))
                                    .add(
                                        Condition::all()
                                            .add(sort_column.eq(dt))
                                            .add(tasks::Column::Id.lt(id)),
                                    ),
                            );
                        } else {
                            query = query.filter(
                                Condition::any()
                                    .add(sort_column.gt(dt))
                                    .add(
                                        Condition::all()
                                            .add(sort_column.eq(dt))
                                            .add(tasks::Column::Id.gt(id)),
                                    ),
                            );
                        }
                    }
                }
            }
        }

        if is_desc {
            query = query.order_by_desc(sort_column).order_by_desc(tasks::Column::Id);
        } else {
            query = query.order_by_asc(sort_column).order_by_asc(tasks::Column::Id);
        }

        let db_tasks = query.limit(fetch_limit).all(&self.db).await?;

        let has_more = db_tasks.len() > limit as usize;
        let tasks: Vec<Task> = db_tasks
            .into_iter()
            .take(limit as usize)
            .map(model_to_task)
            .collect();

        let next_cursor = if has_more {
            if let Some(last) = tasks.last() {
                let value = match sort_by {
                    "created_at" => last.created_at.timestamp().to_string(),
                    "updated_at" => last.updated_at.timestamp().to_string(),
                    "priority" => last.priority.as_i32().to_string(),
                    "status" => last.status.as_str().to_string(),
                    _ => last.created_at.timestamp().to_string(),
                };
                Some(encode_cursor(&value, last.id))
            } else {
                None
            }
        } else {
            None
        };

        let count_query = tasks::Entity::find();
        let count_query = if let Some(s) = params.status {
            count_query.filter(tasks::Column::Status.eq(s.as_str()))
        } else {
            count_query
        };
        let total = count_query.count(&self.db).await?;

        Ok(TaskListResult {
            tasks,
            total,
            next_cursor,
            has_more,
        })
    }

    async fn count(&self, status: Option<TaskStatus>) -> Result<u64> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        Ok(query.count(&self.db).await?)
    }

    async fn update_status(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
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

    async fn set_claimed_by(&self, task_id: &Uuid, worker_id: Option<Uuid>) -> ServerResult<()> {
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

    async fn complete_iteration(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
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

    async fn set_review_data(&self, task_id: &Uuid, review_data: ReviewData) -> ServerResult<()> {
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

    async fn get_review_data(&self, task_id: &Uuid) -> ServerResult<Option<ReviewData>> {
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        match task.review_data_json {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    async fn get_next_queued(&self) -> Result<Option<Task>> {
        let task = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Queued.as_str()))
            .order_by_desc(tasks::Column::Priority)
            .order_by_asc(tasks::Column::CreatedAt)
            .one(&self.db)
            .await?;

        Ok(task.map(model_to_task))
    }

    async fn requeue_task(&self, task_id: &Uuid) -> ServerResult<()> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let current_status = TaskStatus::from_str(&task.status).unwrap_or(TaskStatus::Created);

        if !matches!(
            current_status,
            TaskStatus::InProgress | TaskStatus::Claimed | TaskStatus::AwaitingReview
        ) {
            return Ok(());
        }

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(TaskStatus::Queued.as_str().to_string());
        task.claimed_by = Set(None);
        task.updated_at = Set(now);
        task.update(&self.db).await?;

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
        let non_terminal_statuses = vec![
            TaskStatus::InProgress.as_str(),
            TaskStatus::Claimed.as_str(),
            TaskStatus::AwaitingReview.as_str(),
            TaskStatus::PendingRework.as_str(),
        ];

        let db_tasks = tasks::Entity::find()
            .filter(tasks::Column::Status.is_in(non_terminal_statuses))
            .filter(tasks::Column::ClaimedBy.is_null())
            .all(&self.db)
            .await?;

        Ok(db_tasks.into_iter().map(model_to_task).collect())
    }

    async fn find_timed_out_tasks(&self) -> ServerResult<Vec<Task>> {
        let now = Utc::now();
        let active_statuses = vec![
            TaskStatus::InProgress.as_str(),
            TaskStatus::Claimed.as_str(),
        ];

        let db_tasks = tasks::Entity::find()
            .filter(tasks::Column::Status.is_in(active_statuses))
            .all(&self.db)
            .await?;

        let result: Vec<Task> = db_tasks
            .into_iter()
            .filter(|t| {
                let elapsed = (now - t.created_at).num_seconds();
                elapsed > t.max_execution_time
            })
            .map(model_to_task)
            .collect();

        Ok(result)
    }

    async fn fail_task(&self, task_id: &Uuid, _reason: &str) -> ServerResult<()> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(TaskStatus::Failed.as_str().to_string());
        task.claimed_by = Set(None);
        task.updated_at = Set(now);
        task.update(&self.db).await?;

        Ok(())
    }

    async fn retry_task(&self, task_id: &Uuid, clear_review_data: bool) -> ServerResult<Task> {
        let now = Utc::now();
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::TaskNotFound(*task_id))?;

        let current_status = TaskStatus::from_str(&task.status).unwrap_or(TaskStatus::Created);

        if !matches!(
            current_status,
            TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::Completed
        ) {
            return Err(ServerError::InvalidStatus(format!(
                "Task with status '{}' cannot be retried. Only Failed, Cancelled, or Completed tasks can be retried.",
                current_status.as_str()
            )));
        }

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(TaskStatus::Queued.as_str().to_string());
        task.claimed_by = Set(None);
        task.updated_at = Set(now);
        
        if clear_review_data {
            task.review_data_json = Set(None);
        }

        let updated = task.update(&self.db).await?;

        Ok(model_to_task(updated))
    }
}
