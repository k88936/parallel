use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use std::collections::HashMap;
use uuid::Uuid;

use parallel_common::task::Task;
use parallel_common::{ReviewData, TaskPriority, TaskStatus};

use crate::db::entity::{NewTask, Task as DbTask};
use crate::db::schema::tasks as tasks_schema;
use crate::errors::{ServerError, ServerResult};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct TaskRepository {
    pool: DbPool,
}

impl TaskRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> ServerResult<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>> {
        self.pool.get().map_err(|e| ServerError::InternalError(e.to_string()))
    }
}

fn db_task_to_task(t: DbTask) -> Task {
    let DbTask {
        id,
        title,
        repo_url,
        description,
        base_branch,
        target_branch,
        status,
        priority,
        created_at,
        updated_at,
        claimed_by,
        ssh_key,
        max_execution_time,
        required_labels_json,
        ..
    } = t;
    
    Task {
        id: Uuid::parse_str(&id).unwrap_or_default(),
        title,
        repo_url,
        description,
        base_branch,
        target_branch,
        status: TaskStatus::from_str(&status).unwrap_or(TaskStatus::Created),
        priority: TaskPriority::from_i32(priority).unwrap_or(TaskPriority::Normal),
        created_at: chrono::DateTime::from_naive_utc_and_offset(created_at, Utc),
        updated_at: chrono::DateTime::from_naive_utc_and_offset(updated_at, Utc),
        claimed_by: claimed_by.as_ref().and_then(|s| Uuid::parse_str(s).ok()),
        ssh_key,
        max_execution_time,
        required_labels: serde_json::from_str(&required_labels_json).unwrap_or_default(),
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
pub trait TaskRepositoryTrait: Send + Sync {
    async fn create(
        &self,
        id: Uuid,
        title: String,
        repo_url: String,
        description: String,
        base_branch: String,
        target_branch: String,
        status: TaskStatus,
        priority: TaskPriority,
        ssh_key: String,
        max_execution_time: i64,
        project_id: Option<Uuid>,
        required_labels: HashMap<String, String>,
    ) -> Result<()>;

    async fn find_by_id(&self, task_id: &Uuid) -> ServerResult<Task>;

    async fn find_many(
        &self,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
        repo_url: Option<&str>,
        worker_id: Option<Uuid>,
        search: Option<&str>,
        created_after: Option<chrono::DateTime<Utc>>,
        created_before: Option<chrono::DateTime<Utc>>,
        project_id: Option<Uuid>,
        sort_by: &str,
        sort_direction: &str,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<(Vec<Task>, bool)>;

    async fn count(&self, status: Option<TaskStatus>) -> Result<u64>;

    async fn update_status(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()>;

    async fn set_claimed_by(&self, task_id: &Uuid, worker_id: Option<Uuid>) -> ServerResult<()>;

    async fn complete_iteration(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()>;

    async fn set_review_data(&self, task_id: &Uuid, status: TaskStatus, review_data: &ReviewData) -> ServerResult<()>;

    async fn get_review_data(&self, task_id: &Uuid) -> ServerResult<Option<ReviewData>>;

    async fn find_next_queued(&self) -> Result<Option<Task>>;

    async fn requeue(&self, task_id: &Uuid) -> ServerResult<Task>;

    async fn find_orphaned(&self) -> ServerResult<Vec<Task>>;

    async fn find_timed_out(&self) -> ServerResult<Vec<Task>>;

    async fn fail(&self, task_id: &Uuid) -> ServerResult<()>;
}

#[async_trait]
impl TaskRepositoryTrait for TaskRepository {
    async fn create(
        &self,
        id: Uuid,
        title: String,
        repo_url: String,
        description: String,
        base_branch: String,
        target_branch: String,
        status: TaskStatus,
        priority: TaskPriority,
        ssh_key: String,
        max_execution_time: i64,
        project_id: Option<Uuid>,
        required_labels: HashMap<String, String>,
    ) -> Result<()> {
        let now = Utc::now();
        let now_naive = now.naive_utc();

        let new_task = NewTask {
            id: id.to_string(),
            title,
            repo_url,
            description,
            base_branch,
            target_branch,
            status: status.as_str().to_string(),
            priority: priority.as_i32(),
            created_at: now_naive,
            updated_at: now_naive,
            claimed_by: None,
            review_data_json: None,
            ssh_key,
            max_execution_time,
            project_id: project_id.map(|p| p.to_string()),
            required_labels_json: serde_json::to_string(&required_labels)?,
        };

        let mut conn = self.get_conn()?;
        diesel::insert_into(tasks_schema::table)
            .values(&new_task)
            .execute(&mut conn)?;

        Ok(())
    }

    async fn find_by_id(&self, task_id: &Uuid) -> ServerResult<Task> {
        let mut conn = self.get_conn()?;
        let task = tasks_schema::table
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .first::<DbTask>(&mut conn)
            .map_err(|_| ServerError::TaskNotFound(*task_id))?;

        Ok(db_task_to_task(task))
    }

    async fn find_many(
        &self,
        status: Option<TaskStatus>,
        priority: Option<TaskPriority>,
        repo_url: Option<&str>,
        worker_id: Option<Uuid>,
        search: Option<&str>,
        created_after: Option<chrono::DateTime<Utc>>,
        created_before: Option<chrono::DateTime<Utc>>,
        project_id: Option<Uuid>,
        sort_by: &str,
        sort_direction: &str,
        cursor: Option<&str>,
        limit: u64,
    ) -> Result<(Vec<Task>, bool)> {
        let fetch_limit = limit + 1;

        let mut conn = self.get_conn()?;
        let mut query = tasks_schema::table.into_boxed();

        if let Some(s) = status {
            query = query.filter(tasks_schema::status.eq(s.as_str()));
        }

        if let Some(p) = priority {
            query = query.filter(tasks_schema::priority.eq(p.as_i32()));
        }

        if let Some(repo) = repo_url {
            query = query.filter(tasks_schema::repo_url.eq(repo));
        }

        if let Some(wid) = worker_id {
            query = query.filter(tasks_schema::claimed_by.eq(wid.to_string()));
        }

        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            let title_filter = tasks_schema::title.like(pattern.clone());
            let desc_filter = tasks_schema::description.like(pattern);
            query = query.filter(title_filter.or(desc_filter));
        }

        if let Some(after) = created_after {
            query = query.filter(tasks_schema::created_at.ge(after.naive_utc()));
        }

        if let Some(before) = created_before {
            query = query.filter(tasks_schema::created_at.le(before.naive_utc()));
        }

        if let Some(pid) = project_id {
            query = query.filter(tasks_schema::project_id.eq(pid.to_string()));
        }

        if let Some(c) = cursor {
            if let Some((value, id)) = decode_cursor(c) {
                let is_desc = sort_direction == "desc";
                
                match sort_by {
                    "created_at" | "updated_at" => {
                        if let Ok(ts) = value.parse::<i64>() {
                            let dt = DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()).naive_utc();
                            if is_desc {
                                query = query.filter(
                                    tasks_schema::created_at.lt(dt)
                                        .or(tasks_schema::created_at.eq(dt).and(tasks_schema::id.lt(id.to_string())))
                                );
                            } else {
                                query = query.filter(
                                    tasks_schema::created_at.gt(dt)
                                        .or(tasks_schema::created_at.eq(dt).and(tasks_schema::id.gt(id.to_string())))
                                );
                            }
                        }
                    }
                    "priority" => {
                        if let Ok(p) = value.parse::<i32>() {
                            if is_desc {
                                query = query.filter(
                                    tasks_schema::priority.lt(p)
                                        .or(tasks_schema::priority.eq(p).and(tasks_schema::id.lt(id.to_string())))
                                );
                            } else {
                                query = query.filter(
                                    tasks_schema::priority.gt(p)
                                        .or(tasks_schema::priority.eq(p).and(tasks_schema::id.gt(id.to_string())))
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let is_desc = sort_direction == "desc";
        
        match sort_by {
            "created_at" => {
                if is_desc {
                    query = query.order_by(tasks_schema::created_at.desc())
                        .order_by(tasks_schema::id.desc());
                } else {
                    query = query.order_by(tasks_schema::created_at.asc())
                        .order_by(tasks_schema::id.asc());
                }
            }
            "updated_at" => {
                if is_desc {
                    query = query.order_by(tasks_schema::updated_at.desc())
                        .order_by(tasks_schema::id.desc());
                } else {
                    query = query.order_by(tasks_schema::updated_at.asc())
                        .order_by(tasks_schema::id.asc());
                }
            }
            "priority" => {
                if is_desc {
                    query = query.order_by(tasks_schema::priority.desc())
                        .order_by(tasks_schema::id.desc());
                } else {
                    query = query.order_by(tasks_schema::priority.asc())
                        .order_by(tasks_schema::id.asc());
                }
            }
            _ => {
                query = query.order_by(tasks_schema::created_at.desc())
                    .order_by(tasks_schema::id.desc());
            }
        }

        let db_tasks = query
            .limit(fetch_limit as i64)
            .load::<DbTask>(&mut conn)?;

        let has_more = db_tasks.len() > limit as usize;
        let tasks: Vec<Task> = db_tasks
            .into_iter()
            .take(limit as usize)
            .map(db_task_to_task)
            .collect();

        Ok((tasks, has_more))
    }

    async fn count(&self, status: Option<TaskStatus>) -> Result<u64> {
        let mut conn = self.get_conn()?;
        
        let count = if let Some(s) = status {
            tasks_schema::table
                .filter(tasks_schema::status.eq(s.as_str()))
                .count()
                .get_result::<i64>(&mut conn)? as u64
        } else {
            tasks_schema::table
                .count()
                .get_result::<i64>(&mut conn)? as u64
        };

        Ok(count)
    }

    async fn update_status(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(tasks_schema::table)
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .set((
                tasks_schema::status.eq(status.as_str()),
                tasks_schema::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::TaskNotFound(*task_id));
        }

        Ok(())
    }

    async fn set_claimed_by(&self, task_id: &Uuid, worker_id: Option<Uuid>) -> ServerResult<()> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(tasks_schema::table)
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .set((
                tasks_schema::claimed_by.eq(worker_id.map(|w| w.to_string())),
                tasks_schema::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::TaskNotFound(*task_id));
        }

        Ok(())
    }

    async fn complete_iteration(&self, task_id: &Uuid, status: TaskStatus) -> ServerResult<()> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        
        if status == TaskStatus::Completed || status == TaskStatus::Cancelled {
            let rows_affected = diesel::update(tasks_schema::table)
                .filter(tasks_schema::id.eq(task_id.to_string()))
                .set((
                    tasks_schema::status.eq(status.as_str()),
                    tasks_schema::claimed_by.eq::<Option<String>>(None),
                    tasks_schema::updated_at.eq(now),
                ))
                .execute(&mut conn)?;

            if rows_affected == 0 {
                return Err(ServerError::TaskNotFound(*task_id));
            }
        } else {
            let rows_affected = diesel::update(tasks_schema::table)
                .filter(tasks_schema::id.eq(task_id.to_string()))
                .set((
                    tasks_schema::status.eq(status.as_str()),
                    tasks_schema::updated_at.eq(now),
                ))
                .execute(&mut conn)?;

            if rows_affected == 0 {
                return Err(ServerError::TaskNotFound(*task_id));
            }
        }

        Ok(())
    }

    async fn set_review_data(&self, task_id: &Uuid, status: TaskStatus, review_data: &ReviewData) -> ServerResult<()> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(tasks_schema::table)
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .set((
                tasks_schema::status.eq(status.as_str()),
                tasks_schema::review_data_json.eq(Some(serde_json::to_string(review_data)?)),
                tasks_schema::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::TaskNotFound(*task_id));
        }

        Ok(())
    }

    async fn get_review_data(&self, task_id: &Uuid) -> ServerResult<Option<ReviewData>> {
        let mut conn = self.get_conn()?;
        let task = tasks_schema::table
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .first::<DbTask>(&mut conn)
            .map_err(|_| ServerError::TaskNotFound(*task_id))?;

        match task.review_data_json {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    async fn find_next_queued(&self) -> Result<Option<Task>> {
        let mut conn = self.get_conn()?;
        let task = tasks_schema::table
            .filter(tasks_schema::status.eq(TaskStatus::Queued.as_str()))
            .order_by(tasks_schema::priority.desc())
            .order_by(tasks_schema::created_at.asc())
            .first::<DbTask>(&mut conn)
            .optional()?;

        Ok(task.map(db_task_to_task))
    }

    async fn requeue(&self, task_id: &Uuid) -> ServerResult<Task> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        
        let rows_affected = diesel::update(tasks_schema::table)
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .set((
                tasks_schema::status.eq(TaskStatus::Queued.as_str()),
                tasks_schema::claimed_by.eq::<Option<String>>(None),
                tasks_schema::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::TaskNotFound(*task_id));
        }

        let task = tasks_schema::table
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .first::<DbTask>(&mut conn)?;

        Ok(db_task_to_task(task))
    }

    async fn find_orphaned(&self) -> ServerResult<Vec<Task>> {
        let non_terminal_statuses = vec![
            TaskStatus::InProgress.as_str(),
            TaskStatus::Claimed.as_str(),
            TaskStatus::AwaitingReview.as_str(),
            TaskStatus::PendingResponse.as_str(),
        ];

        let mut conn = self.get_conn()?;
        let db_tasks = tasks_schema::table
            .filter(tasks_schema::status.eq_any(non_terminal_statuses))
            .filter(tasks_schema::claimed_by.is_null())
            .load::<DbTask>(&mut conn)?;

        Ok(db_tasks.into_iter().map(db_task_to_task).collect())
    }

    async fn find_timed_out(&self) -> ServerResult<Vec<Task>> {
        let now = Utc::now();
        let active_statuses = vec![
            TaskStatus::InProgress.as_str(),
            TaskStatus::Claimed.as_str(),
            TaskStatus::PendingResponse.as_str(),
        ];

        let mut conn = self.get_conn()?;
        let db_tasks = tasks_schema::table
            .filter(tasks_schema::status.eq_any(active_statuses))
            .load::<DbTask>(&mut conn)?;

        let result: Vec<Task> = db_tasks
            .into_iter()
            .filter(|t| {
                let task_dt = chrono::DateTime::from_naive_utc_and_offset(t.created_at, Utc);
                let elapsed = (now - task_dt).num_seconds();
                elapsed > t.max_execution_time
            })
            .map(db_task_to_task)
            .collect();

        Ok(result)
    }

    async fn fail(&self, task_id: &Uuid) -> ServerResult<()> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(tasks_schema::table)
            .filter(tasks_schema::id.eq(task_id.to_string()))
            .set((
                tasks_schema::status.eq(TaskStatus::Failed.as_str()),
                tasks_schema::claimed_by.eq::<Option<String>>(None),
                tasks_schema::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::TaskNotFound(*task_id));
        }

        Ok(())
    }
}

impl TaskRepository {
    pub fn encode_cursor_for_task(task: &Task, sort_by: &str) -> Option<String> {
        let value = match sort_by {
            "created_at" => task.created_at.timestamp().to_string(),
            "updated_at" => task.updated_at.timestamp().to_string(),
            "priority" => task.priority.as_i32().to_string(),
            "status" => task.status.as_str().to_string(),
            _ => task.created_at.timestamp().to_string(),
        };
        Some(encode_cursor(&value, task.id))
    }
}
