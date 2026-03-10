use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use uuid::Uuid;

use parallel_common::{WorkerCapabilities, WorkerInfo, WorkerStatus};

use super::task_repository::DbPool;
use crate::db::entity::{NewWorker, Worker as DbWorker};
use crate::db::schema::workers as workers_schema;
use crate::errors::{ServerError, ServerResult};

pub struct WorkerRepository {
    pool: DbPool,
}

impl WorkerRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn get_conn(&self) -> ServerResult<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>> {
        self.pool.get().map_err(|e| ServerError::InternalError(e.to_string()))
    }
}

fn db_worker_to_worker_info(w: DbWorker) -> ServerResult<WorkerInfo> {
    Ok(WorkerInfo {
        id: w.get_uuid(),
        token: w.token,
        name: w.name,
        status: WorkerStatus::from_str(&w.status).unwrap_or(WorkerStatus::Offline),
        last_heartbeat: chrono::DateTime::from_naive_utc_and_offset(w.last_heartbeat, Utc),
        current_tasks: serde_json::from_str(&w.current_tasks_json)?,
        capabilities: serde_json::from_str(&w.capabilities_json)?,
        max_concurrent: w.max_concurrent as usize,
    })
}

#[async_trait]
pub trait WorkerRepositoryTrait: Send + Sync {
    async fn create(
        &self,
        id: Uuid,
        token: String,
        name: String,
        capabilities: &WorkerCapabilities,
        max_concurrent: usize,
    ) -> ServerResult<WorkerInfo>;

    async fn find_by_id(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo>;

    async fn find_by_token(&self, token: &str) -> ServerResult<WorkerInfo>;

    async fn find_all(&self) -> ServerResult<Vec<WorkerInfo>>;

    async fn update_heartbeat(
        &self,
        worker_id: &Uuid,
        running_tasks: Vec<Uuid>,
        status: WorkerStatus,
    ) -> ServerResult<()>;

    async fn add_task(&self, worker_id: &Uuid, task_id: &Uuid) -> ServerResult<()>;

    async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool>;

    async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>>;

    async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()>;

    async fn find_stale(&self, timeout_seconds: i64) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>>;

    async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()>;
}

#[async_trait]
impl WorkerRepositoryTrait for WorkerRepository {
    async fn create(
        &self,
        id: Uuid,
        token: String,
        name: String,
        capabilities: &WorkerCapabilities,
        max_concurrent: usize,
    ) -> ServerResult<WorkerInfo> {
        let now = Utc::now().naive_utc();
        let capabilities_json = serde_json::to_string(capabilities)?;

        let new_worker = NewWorker {
            id: id.to_string(),
            token: token.clone(),
            name: name.clone(),
            status: WorkerStatus::Idle.as_str().to_string(),
            last_heartbeat: now,
            current_tasks_json: "[]".to_string(),
            pending_instructions_json: "[]".to_string(),
            capabilities_json,
            max_concurrent: max_concurrent as i32,
        };

        let mut conn = self.get_conn()?;
        diesel::insert_into(workers_schema::table)
            .values(&new_worker)
            .execute(&mut conn)?;

        Ok(WorkerInfo {
            id,
            token,
            name,
            status: WorkerStatus::Idle,
            last_heartbeat: chrono::DateTime::from_naive_utc_and_offset(now, Utc),
            current_tasks: vec![],
            capabilities: capabilities.clone(),
            max_concurrent,
        })
    }

    async fn find_by_id(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo> {
        let mut conn = self.get_conn()?;
        let worker = workers_schema::table
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .first::<DbWorker>(&mut conn)
            .map_err(|_| ServerError::WorkerNotFound(*worker_id))?;

        db_worker_to_worker_info(worker)
    }

    async fn find_by_token(&self, token: &str) -> ServerResult<WorkerInfo> {
        let mut conn = self.get_conn()?;
        let worker = workers_schema::table
            .filter(workers_schema::token.eq(token))
            .first::<DbWorker>(&mut conn)
            .map_err(|_| ServerError::InvalidToken)?;

        db_worker_to_worker_info(worker)
    }

    async fn find_all(&self) -> ServerResult<Vec<WorkerInfo>> {
        let mut conn = self.get_conn()?;
        let workers = workers_schema::table
            .load::<DbWorker>(&mut conn)?;

        workers.into_iter().map(db_worker_to_worker_info).collect()
    }

    async fn update_heartbeat(
        &self,
        worker_id: &Uuid,
        running_tasks: Vec<Uuid>,
        status: WorkerStatus,
    ) -> ServerResult<()> {
        let now = Utc::now().naive_utc();
        
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(workers_schema::table)
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .set((
                workers_schema::last_heartbeat.eq(now),
                workers_schema::current_tasks_json.eq(serde_json::to_string(&running_tasks)?),
                workers_schema::status.eq(status.as_str()),
            ))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::WorkerNotFound(*worker_id));
        }

        Ok(())
    }

    async fn add_task(&self, worker_id: &Uuid, task_id: &Uuid) -> ServerResult<()> {
        let mut conn = self.get_conn()?;
        let worker = workers_schema::table
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .first::<DbWorker>(&mut conn)
            .map_err(|_| ServerError::WorkerNotFound(*worker_id))?;

        let mut running_tasks: Vec<String> = serde_json::from_str(&worker.current_tasks_json)?;
        running_tasks.push(task_id.to_string());

        diesel::update(workers_schema::table)
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .set((
                workers_schema::current_tasks_json.eq(serde_json::to_string(&running_tasks)?),
                workers_schema::status.eq(WorkerStatus::Busy.as_str()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool> {
        let mut conn = self.get_conn()?;
        let worker = workers_schema::table
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .first::<DbWorker>(&mut conn)
            .map_err(|_| ServerError::WorkerNotFound(*worker_id))?;

        let running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)?;
        Ok(running_tasks.len() < worker.max_concurrent as usize)
    }

    async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>> {
        let mut conn = self.get_conn()?;
        let worker = workers_schema::table
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .first::<DbWorker>(&mut conn)
            .map_err(|_| ServerError::WorkerNotFound(*worker_id))?;

        Ok(serde_json::from_str(&worker.current_tasks_json)?)
    }

    async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()> {
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(workers_schema::table)
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .set(workers_schema::status.eq(status.as_str()))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::WorkerNotFound(*worker_id));
        }

        Ok(())
    }

    async fn find_stale(&self, timeout_seconds: i64) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>> {
        let cutoff = Utc::now() - chrono::Duration::seconds(timeout_seconds);
        let cutoff_naive = cutoff.naive_utc();

        let mut conn = self.get_conn()?;
        let stale_workers = workers_schema::table
            .filter(workers_schema::last_heartbeat.lt(cutoff_naive))
            .filter(workers_schema::status.ne(WorkerStatus::Offline.as_str()))
            .filter(workers_schema::status.ne(WorkerStatus::Dead.as_str()))
            .load::<DbWorker>(&mut conn)?;

        let result: Vec<(Uuid, Vec<Uuid>)> = stale_workers
            .into_iter()
            .map(|w| {
                let tasks: Vec<Uuid> =
                    serde_json::from_str(&w.current_tasks_json).unwrap_or_default();
                (w.get_uuid(), tasks)
            })
            .collect();

        Ok(result)
    }

    async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()> {
        let mut conn = self.get_conn()?;
        let rows_affected = diesel::update(workers_schema::table)
            .filter(workers_schema::id.eq(worker_id.to_string()))
            .set(workers_schema::current_tasks_json.eq("[]"))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(ServerError::WorkerNotFound(*worker_id));
        }

        Ok(())
    }
}
