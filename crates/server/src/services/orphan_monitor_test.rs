#[cfg(test)]
mod tests {
    use crate::errors::{ServerError, ServerResult};
    use async_trait::async_trait;
    use chrono::Utc;
    use parallel_protocol::{Task, TaskPriority, TaskStatus, WorkerCapabilities, WorkerInfo, WorkerStatus};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;
    use crate::services::orphan_monitor::OrphanMonitor;
    use crate::services::{TaskServiceTrait, WorkerServiceTrait};

    struct MockTaskService {
        orphaned_tasks: Arc<Mutex<Vec<Task>>>,
        timed_out_tasks: Arc<Mutex<Vec<Task>>>,
        requeued_tasks: Arc<Mutex<Vec<Uuid>>>,
        failed_tasks: Arc<Mutex<Vec<Uuid>>>,
    }

    impl MockTaskService {
        fn new() -> Self {
            Self {
                orphaned_tasks: Arc::new(Mutex::new(Vec::new())),
                timed_out_tasks: Arc::new(Mutex::new(Vec::new())),
                requeued_tasks: Arc::new(Mutex::new(Vec::new())),
                failed_tasks: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn add_orphaned_task(&self, task: Task) {
            self.orphaned_tasks.lock().unwrap().push(task);
        }

        fn add_timed_out_task(&self, task: Task) {
            self.timed_out_tasks.lock().unwrap().push(task);
        }

        fn get_requeued_tasks(&self) -> Vec<Uuid> {
            self.requeued_tasks.lock().unwrap().clone()
        }

        fn get_failed_tasks(&self) -> Vec<Uuid> {
            self.failed_tasks.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl TaskServiceTrait for MockTaskService {
        async fn create(
            &self,
            _repo_url: String,
            _description: String,
            _base_branch: String,
            _target_branch: String,
            _priority: TaskPriority,
            _ssh_key: String,
            _max_execution_time: i64,
        ) -> anyhow::Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn get(&self, _task_id: &Uuid) -> ServerResult<Task> {
            Err(ServerError::TaskNotFound(Uuid::nil()))
        }

        async fn list(
            &self,
            _status: Option<TaskStatus>,
            _limit: Option<u64>,
            _offset: Option<u64>,
        ) -> anyhow::Result<Vec<Task>> {
            Ok(Vec::new())
        }

        async fn count(&self, _status: Option<TaskStatus>) -> anyhow::Result<u64> {
            Ok(0)
        }

        async fn update_status(
            &self,
            _task_id: &Uuid,
            _status: TaskStatus,
        ) -> ServerResult<()> {
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
    }

    struct MockWorkerService {
        workers: Arc<Mutex<HashMap<Uuid, WorkerInfo>>>,
    }

    impl MockWorkerService {
        fn new() -> Self {
            Self {
                workers: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn add_worker(&self, worker: WorkerInfo) {
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
                name: "test".to_string(),
                status: WorkerStatus::Idle,
                last_heartbeat: Utc::now(),
                current_tasks: vec![],
                capabilities: WorkerCapabilities::default(),
                max_concurrent: 1,
            })
        }

        async fn get(&self, worker_id: &Uuid) -> ServerResult<WorkerInfo> {
            self.workers
                .lock()
                .unwrap()
                .get(worker_id)
                .cloned()
                .ok_or(ServerError::WorkerNotFound(*worker_id))
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
            _worker_id: &Uuid,
            _status: WorkerStatus,
        ) -> ServerResult<()> {
            Ok(())
        }

        async fn find_stale_workers(
            &self,
            _timeout_seconds: i64,
        ) -> ServerResult<Vec<(Uuid, Vec<Uuid>)>> {
            Ok(Vec::new())
        }

        async fn clear_tasks(&self, _worker_id: &Uuid) -> ServerResult<()> {
            Ok(())
        }
    }

    fn create_test_task(id: Uuid, claimed_by: Option<Uuid>) -> Task {
        Task {
            id,
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

    fn create_test_worker(id: Uuid, status: WorkerStatus) -> WorkerInfo {
        WorkerInfo {
            id,
            name: "test-worker".to_string(),
            status,
            last_heartbeat: Utc::now(),
            current_tasks: vec![],
            capabilities: WorkerCapabilities::default(),
            max_concurrent: 1,
        }
    }

    #[tokio::test]
    async fn test_check_orphans_no_tasks() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let monitor = OrphanMonitor::new(task_service, worker_service, 60);

        let result = monitor.check_orphans().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_orphans_with_unclaimed_task() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let task_id = Uuid::new_v4();
        let task = create_test_task(task_id, None);
        task_service.add_orphaned_task(task);

        let monitor = OrphanMonitor::new(task_service.clone(), worker_service, 60);

        let result = monitor.check_orphans().await;
        assert!(result.is_ok());

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 1);
        assert_eq!(requeued[0], task_id);
    }

    #[tokio::test]
    async fn test_check_orphans_with_offline_worker() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        let worker = create_test_worker(worker_id, WorkerStatus::Offline);
        worker_service.add_worker(worker);

        let task_id = Uuid::new_v4();
        let task = create_test_task(task_id, Some(worker_id));
        task_service.add_orphaned_task(task);

        let monitor = OrphanMonitor::new(task_service.clone(), worker_service.clone(), 60);

        let result = monitor.check_orphans().await;
        assert!(result.is_ok());

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 1);
        assert_eq!(requeued[0], task_id);
    }

    #[tokio::test]
    async fn test_check_orphans_with_online_worker() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        let worker = create_test_worker(worker_id, WorkerStatus::Idle);
        worker_service.add_worker(worker);

        let task_id = Uuid::new_v4();
        let task = create_test_task(task_id, Some(worker_id));
        task_service.add_orphaned_task(task);

        let monitor = OrphanMonitor::new(task_service.clone(), worker_service.clone(), 60);

        let result = monitor.check_orphans().await;
        assert!(result.is_ok());

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 0);
    }

    #[tokio::test]
    async fn test_check_orphans_with_unknown_worker() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let task = create_test_task(task_id, Some(worker_id));
        task_service.add_orphaned_task(task);

        let monitor = OrphanMonitor::new(task_service.clone(), worker_service, 60);

        let result = monitor.check_orphans().await;
        assert!(result.is_ok());

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 1);
        assert_eq!(requeued[0], task_id);
    }

    #[tokio::test]
    async fn test_check_timeouts_no_tasks() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let monitor = OrphanMonitor::new(task_service, worker_service, 60);

        let result = monitor.check_timeouts().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_timeouts_with_timed_out_task() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let task_id = Uuid::new_v4();
        let task = create_test_task(task_id, None);
        task_service.add_timed_out_task(task);

        let monitor = OrphanMonitor::new(task_service.clone(), worker_service, 60);

        let result = monitor.check_timeouts().await;
        assert!(result.is_ok());

        let failed = task_service.get_failed_tasks();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0], task_id);
    }

    #[tokio::test]
    async fn test_check_timeouts_multiple_tasks() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        let task1 = create_test_task(task_id1, None);
        let task2 = create_test_task(task_id2, None);

        task_service.add_timed_out_task(task1);
        task_service.add_timed_out_task(task2);

        let monitor = OrphanMonitor::new(task_service.clone(), worker_service, 60);

        let result = monitor.check_timeouts().await;
        assert!(result.is_ok());

        let failed = task_service.get_failed_tasks();
        assert_eq!(failed.len(), 2);
        assert!(failed.contains(&task_id1));
        assert!(failed.contains(&task_id2));
    }
}
