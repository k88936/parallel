use chrono::Utc;
use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{WorkerCapabilities, WorkerInfo, WorkerStatus};

use crate::db::entity::workers;
use crate::errors::{ServerError, ServerResult};

pub struct WorkerService {
    db: DatabaseConnection,
}

impl WorkerService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn register(
        &self,
        name: String,
        capabilities: WorkerCapabilities,
        max_concurrent: usize,
    ) -> ServerResult<WorkerInfo> {
        let worker_id = Uuid::new_v4();
        let now = Utc::now();
        let capabilities_json = serde_json::to_string(&capabilities)?;

        let worker = workers::ActiveModel {
            id: Set(worker_id),
            name: Set(name.clone()),
            status: Set(WorkerStatus::Idle.as_str().to_string()),
            last_heartbeat: Set(now),
            current_tasks_json: Set("[]".to_string()),
            pending_instructions_json: Set("[]".to_string()),
            capabilities_json: Set(capabilities_json),
            max_concurrent: Set(max_concurrent as i32),
        };

        workers::Entity::insert(worker)
            .exec(&self.db)
            .await?;

        Ok(WorkerInfo {
            id: worker_id,
            name,
            status: WorkerStatus::Idle,
            last_heartbeat: now,
            current_tasks: vec![],
            capabilities,
            max_concurrent,
        })
    }

    pub async fn get(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        Ok(WorkerInfo {
            id: worker.id,
            name: worker.name,
            status: WorkerStatus::from_str(&worker.status).unwrap_or(WorkerStatus::Offline),
            last_heartbeat: worker.last_heartbeat,
            current_tasks: serde_json::from_str(&worker.current_tasks_json)?,
            capabilities: serde_json::from_str(&worker.capabilities_json)?,
            max_concurrent: worker.max_concurrent as usize,
        })
    }

    pub async fn list(&self) -> ServerResult<Vec<WorkerInfo>> {
        let workers = workers::Entity::find()
            .all(&self.db)
            .await?;

        let worker_infos: Vec<WorkerInfo> = workers
            .into_iter()
            .map(|w| {
                Ok(WorkerInfo {
                    id: w.id,
                    name: w.name,
                    status: WorkerStatus::from_str(&w.status).unwrap_or(WorkerStatus::Offline),
                    last_heartbeat: w.last_heartbeat,
                    current_tasks: serde_json::from_str(&w.current_tasks_json)?,
                    capabilities: serde_json::from_str(&w.capabilities_json)?,
                    max_concurrent: w.max_concurrent as usize,
                })
            })
            .collect::<ServerResult<Vec<_>>>()?;

        Ok(worker_infos)
    }

    pub async fn update_heartbeat(&self, worker_id: &Uuid, running_tasks: Vec<Uuid>) -> ServerResult<()> {
        let now = Utc::now();
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.last_heartbeat = Set(now);
        worker.current_tasks_json = Set(serde_json::to_string(&running_tasks)?);
        worker.status = Set(if running_tasks.is_empty() {
            "idle".to_string()
        } else {
            "busy".to_string()
        });
        worker.update(&self.db).await?;

        Ok(())
    }

    pub async fn add_task(&self, worker_id: &Uuid, task_id: Uuid) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)?;
        running_tasks.push(task_id);

        let mut worker: workers::ActiveModel = worker.into();
        worker.current_tasks_json = Set(serde_json::to_string(&running_tasks)?);
        worker.status = Set("busy".to_string());
        worker.update(&self.db).await?;

        Ok(())
    }

    pub async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)?;
        Ok(running_tasks.len() < worker.max_concurrent as usize)
    }

    pub async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        Ok(serde_json::from_str(&worker.current_tasks_json)?)
    }

    pub async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.status = Set(status.as_str().to_string());
        worker.update(&self.db).await?;

        Ok(())
    }

    pub async fn find_stale_workers(&self, timeout_seconds: i64) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>> {
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
                let tasks: Vec<Uuid> = serde_json::from_str(&w.current_tasks_json).unwrap_or_default();
                (w.id, tasks)
            })
            .collect();

        Ok(result)
    }

    pub async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let mut worker: workers::ActiveModel = worker.into();
        worker.current_tasks_json = Set("[]".to_string());
        worker.update(&self.db).await?;

        Ok(())
    }
}
