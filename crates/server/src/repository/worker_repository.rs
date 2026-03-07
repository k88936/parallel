use async_trait::async_trait;
use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{WorkerCapabilities, WorkerInfo, WorkerStatus};

use crate::db::entity::workers;
use crate::errors::{ServerError, ServerResult};

pub struct WorkerRepository {
    db: DatabaseConnection,
}

impl WorkerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_worker_info(w: workers::Model) -> ServerResult<WorkerInfo> {
    Ok(WorkerInfo {
        id: w.id,
        token: w.token,
        name: w.name,
        status: WorkerStatus::from_str(&w.status).unwrap_or(WorkerStatus::Offline),
        last_heartbeat: w.last_heartbeat,
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

    async fn add_task(&self, worker_id: &Uuid, task_id: Uuid) -> ServerResult<()>;

    async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool>;

    async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>>;

    async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()>;

    async fn find_stale(
        &self,
        timeout_seconds: i64,
    ) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>>;

    async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()>;

    async fn update_pending_instructions(
        &self,
        worker_id: &Uuid,
        instructions: Vec<parallel_protocol::WorkerInstruction>,
    ) -> ServerResult<()>;

    async fn get_pending_instructions(
        &self,
        worker_id: &Uuid,
    ) -> ServerResult<Vec<parallel_protocol::WorkerInstruction>>;
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
        let now = Utc::now();
        let capabilities_json = serde_json::to_string(capabilities)?;

        let worker = workers::ActiveModel {
            id: Set(id),
            token: Set(token.clone()),
            name: Set(name.clone()),
            status: Set(WorkerStatus::Idle.as_str().to_string()),
            last_heartbeat: Set(now),
            current_tasks_json: Set("[]".to_string()),
            pending_instructions_json: Set("[]".to_string()),
            capabilities_json: Set(capabilities_json),
            max_concurrent: Set(max_concurrent as i32),
        };

        workers::Entity::insert(worker).exec(&self.db).await?;

        Ok(WorkerInfo {
            id,
            token,
            name,
            status: WorkerStatus::Idle,
            last_heartbeat: now,
            current_tasks: vec![],
            capabilities: capabilities.clone(),
            max_concurrent,
        })
    }

    async fn find_by_id(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        model_to_worker_info(worker)
    }

    async fn find_by_token(&self, token: &str) -> ServerResult<WorkerInfo> {
        let worker = workers::Entity::find()
            .filter(workers::Column::Token.eq(token))
            .one(&self.db)
            .await?
            .ok_or(ServerError::InvalidToken)?;

        model_to_worker_info(worker)
    }

    async fn find_all(&self) -> ServerResult<Vec<WorkerInfo>> {
        let workers = workers::Entity::find().all(&self.db).await?;

        workers
            .into_iter()
            .map(model_to_worker_info)
            .collect()
    }

    async fn update_heartbeat(
        &self,
        worker_id: &Uuid,
        running_tasks: Vec<Uuid>,
        status: WorkerStatus,
    ) -> ServerResult<()> {
        let now = Utc::now();
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.last_heartbeat = Set(now);
        worker.current_tasks_json = Set(serde_json::to_string(&running_tasks)?);
        worker.status = Set(status.as_str().to_string());
        worker.update(&self.db).await?;

        Ok(())
    }

    async fn add_task(&self, worker_id: &Uuid, task_id: Uuid) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)?;
        running_tasks.push(task_id);

        let mut worker: workers::ActiveModel = worker.into();
        worker.current_tasks_json = Set(serde_json::to_string(&running_tasks)?);
        worker.status = Set(WorkerStatus::Busy.as_str().to_string());
        worker.update(&self.db).await?;

        Ok(())
    }

    async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)?;
        Ok(running_tasks.len() < worker.max_concurrent as usize)
    }

    async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        Ok(serde_json::from_str(&worker.current_tasks_json)?)
    }

    async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.status = Set(status.as_str().to_string());
        worker.update(&self.db).await?;

        Ok(())
    }

    async fn find_stale(
        &self,
        timeout_seconds: i64,
    ) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>> {
        let cutoff = Utc::now() - chrono::Duration::seconds(timeout_seconds);

        let stale_workers = workers::Entity::find()
            .filter(workers::Column::LastHeartbeat.lt(cutoff))
            .filter(workers::Column::Status.ne(WorkerStatus::Offline.as_str()))
            .filter(workers::Column::Status.ne(WorkerStatus::Dead.as_str()))
            .all(&self.db)
            .await?;

        let result: Vec<(Uuid, Vec<Uuid>)> = stale_workers
            .into_iter()
            .map(|w| {
                let tasks: Vec<Uuid> =
                    serde_json::from_str(&w.current_tasks_json).unwrap_or_default();
                (w.id, tasks)
            })
            .collect();

        Ok(result)
    }

    async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.current_tasks_json = Set("[]".to_string());
        worker.update(&self.db).await?;

        Ok(())
    }

    async fn update_pending_instructions(
        &self,
        worker_id: &Uuid,
        instructions: Vec<parallel_protocol::WorkerInstruction>,
    ) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.pending_instructions_json = Set(serde_json::to_string(&instructions)?);
        worker.update(&self.db).await?;

        Ok(())
    }

    async fn get_pending_instructions(
        &self,
        worker_id: &Uuid,
    ) -> ServerResult<Vec<parallel_protocol::WorkerInstruction>> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        Ok(serde_json::from_str(&worker.pending_instructions_json)?)
    }
}
