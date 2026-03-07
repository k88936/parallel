#[cfg(test)]
pub mod mocks {
    use crate::errors::{ServerError, ServerResult};
    use crate::services::traits::{TaskListParams, TaskListResult};
    use crate::services::{TaskServiceTrait, WorkerServiceTrait};
    use async_trait::async_trait;
    use chrono::Utc;
    use parallel_protocol::{
        Task, TaskPriority, TaskStatus, WorkerCapabilities, WorkerInfo, WorkerStatus,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    pub struct MockTaskService {
        pub requeued_tasks: Arc<Mutex<Vec<Uuid>>>,
        pub failed_tasks: Arc<Mutex<Vec<Uuid>>>,
        pub orphaned_tasks: Arc<Mutex<Vec<Task>>>,
        pub timed_out_tasks: Arc<Mutex<Vec<Task>>>,
        pub requeue_should_fail: Arc<Mutex<bool>>,
    }

    impl MockTaskService {
        pub fn new() -> Self {
            Self {
                requeued_tasks: Arc::new(Mutex::new(Vec::new())),
                failed_tasks: Arc::new(Mutex::new(Vec::new())),
                orphaned_tasks: Arc::new(Mutex::new(Vec::new())),
                timed_out_tasks: Arc::new(Mutex::new(Vec::new())),
                requeue_should_fail: Arc::new(Mutex::new(false)),
            }
        }

        pub fn get_requeued_tasks(&self) -> Vec<Uuid> {
            self.requeued_tasks.lock().unwrap().clone()
        }

        pub fn get_failed_tasks(&self) -> Vec<Uuid> {
            self.failed_tasks.lock().unwrap().clone()
        }

        pub fn add_orphaned_task(&self, task: Task) {
            self.orphaned_tasks.lock().unwrap().push(task);
        }

        pub fn add_timed_out_task(&self, task: Task) {
            self.timed_out_tasks.lock().unwrap().push(task);
        }

        pub fn set_requeue_should_fail(&self, should_fail: bool) {
            *self.requeue_should_fail.lock().unwrap() = should_fail;
        }
    }

    #[async_trait]
    impl TaskServiceTrait for MockTaskService {
        async fn create(
            &self,
            _title: String,
            _repo_url: String,
            _description: String,
            _base_branch: String,
            _target_branch: String,
            _priority: TaskPriority,
            _ssh_key: String,
            _max_execution_time: i64,
            project_id: Option<Uuid>,
        ) -> anyhow::Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn get(&self, _task_id: &Uuid) -> ServerResult<Task> {
            Err(ServerError::TaskNotFound(Uuid::nil()))
        }

        async fn list(&self, _params: TaskListParams) -> anyhow::Result<TaskListResult> {
            Ok(TaskListResult {
                tasks: Vec::new(),
                total: 0,
                next_cursor: None,
                has_more: false,
            })
        }

        async fn count(&self, _status: Option<TaskStatus>) -> anyhow::Result<u64> {
            Ok(0)
        }

        async fn update_status(&self, _task_id: &Uuid, _status: TaskStatus) -> ServerResult<()> {
            Ok(())
        }

        async fn set_claimed_by(
            &self,
            _task_id: &Uuid,
            _worker_id: Option<Uuid>,
        ) -> ServerResult<()> {
            Ok(())
        }

        async fn complete_iteration(
            &self,
            _task_id: &Uuid,
            _status: TaskStatus,
        ) -> ServerResult<()> {
            Ok(())
        }

        async fn set_review_data(
            &self,
            _task_id: &Uuid,
            _review_data: parallel_protocol::ReviewData,
        ) -> ServerResult<()> {
            Ok(())
        }

        async fn get_review_data(
            &self,
            _task_id: &Uuid,
        ) -> ServerResult<Option<parallel_protocol::ReviewData>> {
            Ok(None)
        }

        async fn get_next_queued(&self) -> anyhow::Result<Option<Task>> {
            Ok(None)
        }

        async fn requeue_task(&self, task_id: &Uuid) -> ServerResult<()> {
            self.requeued_tasks.lock().unwrap().push(*task_id);
            Ok(())
        }

        async fn requeue_tasks(&self, task_ids: &[Uuid]) -> ServerResult<usize> {
            if *self.requeue_should_fail.lock().unwrap() {
                return Err(ServerError::InternalError("Requeue failed".to_string()));
            }
            let mut count = 0;
            for task_id in task_ids {
                self.requeued_tasks.lock().unwrap().push(*task_id);
                count += 1;
            }
            Ok(count)
        }

        async fn find_orphaned_tasks(&self) -> ServerResult<Vec<Task>> {
            Ok(self.orphaned_tasks.lock().unwrap().clone())
        }

        async fn find_timed_out_tasks(&self) -> ServerResult<Vec<Task>> {
            Ok(self.timed_out_tasks.lock().unwrap().clone())
        }

        async fn fail_task(&self, task_id: &Uuid, _reason: &str) -> ServerResult<()> {
            self.failed_tasks.lock().unwrap().push(*task_id);
            Ok(())
        }

        async fn retry_task(&self, _task_id: &Uuid, _clear_review_data: bool) -> ServerResult<Task> {
            Err(ServerError::TaskNotFound(Uuid::nil()))
        }
    }

    pub struct MockWorkerService {
        pub stale_workers: Arc<Mutex<Vec<(Uuid, Vec<Uuid>)>>>,
        pub worker_statuses: Arc<Mutex<HashMap<Uuid, WorkerStatus>>>,
        pub cleared_tasks: Arc<Mutex<Vec<Uuid>>>,
        pub workers: Arc<Mutex<HashMap<Uuid, WorkerInfo>>>,
        pub update_status_fail_workers: Arc<Mutex<Vec<Uuid>>>,
        pub clear_tasks_should_fail: Arc<Mutex<bool>>,
    }

    impl MockWorkerService {
        pub fn new() -> Self {
            Self {
                stale_workers: Arc::new(Mutex::new(Vec::new())),
                worker_statuses: Arc::new(Mutex::new(HashMap::new())),
                cleared_tasks: Arc::new(Mutex::new(Vec::new())),
                workers: Arc::new(Mutex::new(HashMap::new())),
                update_status_fail_workers: Arc::new(Mutex::new(Vec::new())),
                clear_tasks_should_fail: Arc::new(Mutex::new(false)),
            }
        }

        pub fn add_stale_worker(&self, worker_id: Uuid, running_tasks: Vec<Uuid>) {
            self.stale_workers
                .lock()
                .unwrap()
                .push((worker_id, running_tasks));
        }

        pub fn get_worker_status(&self, worker_id: &Uuid) -> Option<WorkerStatus> {
            self.worker_statuses.lock().unwrap().get(worker_id).cloned()
        }

        pub fn get_cleared_tasks(&self) -> Vec<Uuid> {
            self.cleared_tasks.lock().unwrap().clone()
        }

        pub fn set_update_status_should_fail_for_worker(&self, worker_id: Uuid) {
            self.update_status_fail_workers.lock().unwrap().push(worker_id);
        }

        pub fn set_clear_tasks_should_fail(&self, should_fail: bool) {
            *self.clear_tasks_should_fail.lock().unwrap() = should_fail;
        }

        pub fn add_worker(&self, worker: WorkerInfo) {
            self.workers.lock().unwrap().insert(worker.id, worker);
        }
    }

    #[async_trait]
    impl WorkerServiceTrait for MockWorkerService {
        async fn register(
            &self,
            _name: String,
            _capabilities: WorkerCapabilities,
            _max_concurrent: usize,
        ) -> ServerResult<WorkerInfo> {
            Ok(WorkerInfo {
                id: Uuid::new_v4(),
                token: Uuid::new_v4().to_string(),
                name: "test".to_string(),
                status: WorkerStatus::Idle,
                last_heartbeat: Utc::now(),
                current_tasks: vec![],
                capabilities: WorkerCapabilities::default(),
                max_concurrent: 1,
            })
        }

        async fn get(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo> {
            let workers = self.workers.lock().unwrap();
            if let Some(worker) = workers.get(worker_id) {
                let status = self
                    .worker_statuses
                    .lock()
                    .unwrap()
                    .get(worker_id)
                    .cloned()
                    .unwrap_or_else(|| worker.status.clone());
                let mut worker = worker.clone();
                worker.status = status;
                return Ok(worker);
            }
            drop(workers);
            
            let status = self
                .worker_statuses
                .lock()
                .unwrap()
                .get(worker_id)
                .cloned();
            
            if let Some(status) = status {
                return Ok(WorkerInfo {
                    id: *worker_id,
                    token: Uuid::new_v4().to_string(),
                    name: "test-worker".to_string(),
                    status,
                    last_heartbeat: Utc::now(),
                    current_tasks: vec![],
                    capabilities: WorkerCapabilities::default(),
                    max_concurrent: 1,
                });
            }
            
            Err(ServerError::WorkerNotFound(*worker_id))
        }

        async fn get_by_token(&self, _token: &str) -> ServerResult<WorkerInfo> {
            Err(ServerError::InvalidToken)
        }

        async fn list(&self) -> ServerResult<Vec<WorkerInfo>> {
            Ok(self.workers.lock().unwrap().values().cloned().collect())
        }

        async fn update_heartbeat(
            &self,
            _worker_id: &Uuid,
            _running_tasks: Vec<Uuid>,
        ) -> ServerResult<()> {
            Ok(())
        }

        async fn add_task(&self, _worker_id: &Uuid, _task_id: Uuid) -> ServerResult<()> {
            Ok(())
        }

        async fn has_available_slot(&self, _worker_id: &Uuid) -> ServerResult<bool> {
            Ok(true)
        }

        async fn get_running_tasks(&self, _worker_id: &Uuid) -> ServerResult<Vec<Uuid>> {
            Ok(Vec::new())
        }

        async fn update_status(
            &self,
            worker_id: &Uuid,
            status: WorkerStatus,
        ) -> ServerResult<()> {
            if self.update_status_fail_workers.lock().unwrap().contains(worker_id) {
                return Err(ServerError::InternalError("Update status failed".to_string()));
            }
            self.worker_statuses
                .lock()
                .unwrap()
                .insert(*worker_id, status);
            Ok(())
        }

        async fn find_stale_workers(
            &self,
            _timeout_seconds: i64,
        ) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>> {
            Ok(self.stale_workers.lock().unwrap().clone())
        }

        async fn clear_tasks(&self, worker_id: &Uuid) -> ServerResult<()> {
            if *self.clear_tasks_should_fail.lock().unwrap() {
                return Err(ServerError::InternalError("Clear tasks failed".to_string()));
            }
            self.cleared_tasks.lock().unwrap().push(*worker_id);
            Ok(())
        }
    }

    pub fn create_test_task(id: Uuid, claimed_by: Option<Uuid>) -> Task {
        Task {
            id,
            title: "Test Task".to_string(),
            repo_url: "https://github.com/test/repo".to_string(),
            description: "Test task".to_string(),
            base_branch: "main".to_string(),
            target_branch: "feature/test".to_string(),
            status: TaskStatus::InProgress,
            priority: TaskPriority::Normal,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            claimed_by,
            ssh_key: String::new(),
            max_execution_time: 3600,
        }
    }

    pub fn create_test_worker(id: Uuid, status: WorkerStatus) -> WorkerInfo {
        WorkerInfo {
            id,
            token: Uuid::new_v4().to_string(),
            name: "test-worker".to_string(),
            status,
            last_heartbeat: Utc::now(),
            current_tasks: vec![],
            capabilities: WorkerCapabilities::default(),
            max_concurrent: 1,
        }
    }
}
