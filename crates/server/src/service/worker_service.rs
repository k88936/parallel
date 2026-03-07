use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use parallel_protocol::{WorkerCapabilities, WorkerInfo, WorkerStatus};

use crate::errors::ServerResult;
use crate::repository::{WorkerRepository, WorkerRepositoryTrait};

pub struct WorkerService {
    repository: Arc<WorkerRepository>,
}

impl WorkerService {
    pub fn new(repository: Arc<WorkerRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl WorkerServiceTrait for WorkerService {
    async fn register(
        &self,
        name: String,
        capabilities: WorkerCapabilities,
        max_concurrent: usize,
    ) -> ServerResult<WorkerInfo> {
        let worker_id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();

        self.repository.create(
            worker_id,
            token.clone(),
            name.clone(),
            &capabilities,
            max_concurrent,
        ).await
    }

    async fn get(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo> {
        self.repository.find_by_id(worker_id).await
    }

    async fn get_by_token(&self, token: &str) -> ServerResult<WorkerInfo> {
        self.repository.find_by_token(token).await
    }

    async fn list(&self) -> ServerResult<Vec<WorkerInfo>> {
        self.repository.find_all().await
    }

    async fn update_heartbeat(
        &self,
        worker_id: &Uuid,
        running_tasks: Vec<Uuid>,
    ) -> ServerResult<()> {
        let status = if running_tasks.is_empty() {
            WorkerStatus::Idle
        } else {
            WorkerStatus::Busy
        };
        self.repository.update_heartbeat(worker_id, running_tasks, status).await
    }

    async fn add_task(&self, worker_id: &Uuid, task_id: Uuid) -> ServerResult<()> {
        self.repository.add_task(worker_id, task_id).await
    }

    async fn has_available_slot(&self, worker_id: &Uuid) -> ServerResult<bool> {
        self.repository.has_available_slot(worker_id).await
    }

    async fn get_running_tasks(&self, worker_id: &Uuid) -> ServerResult<Vec<Uuid>> {
        self.repository.get_running_tasks(worker_id).await
    }

    async fn update_status(&self, worker_id: &Uuid, status: WorkerStatus) -> ServerResult<()> {
        self.repository.update_status(worker_id, status).await
    }

    async fn find_stale_workers(
        &self,
        timeout_seconds: i64,
    ) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>> {
        self.repository.find_stale(timeout_seconds).await
    }

    async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()> {
        self.repository.clear_tasks(worker_id).await
    }
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

    async fn get_by_token(&self, token: &str) -> ServerResult<WorkerInfo>;

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