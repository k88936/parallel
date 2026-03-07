#[cfg(test)]
mod tests {
    use crate::errors::{ServerError, ServerResult};
    use crate::services::heartbeat_monitor::HeartbeatMonitor;
    use crate::services::{TaskServiceTrait, WorkerServiceTrait};
    use async_trait::async_trait;
    use chrono::Utc;
    use parallel_protocol::{
        Task, TaskPriority, TaskStatus, WorkerCapabilities, WorkerInfo, WorkerStatus,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;
    use crate::services::traits::{TaskListParams, TaskListResult};

    struct MockTaskService {
        requeued_tasks: Arc<Mutex<Vec<Uuid>>>,
        requeue_should_fail: Arc<Mutex<bool>>,
    }

    impl MockTaskService {
        fn new() -> Self {
            Self {
                requeued_tasks: Arc::new(Mutex::new(Vec::new())),
                requeue_should_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn get_requeued_tasks(&self) -> Vec<Uuid> {
            self.requeued_tasks.lock().unwrap().clone()
        }

        fn set_requeue_should_fail(&self, should_fail: bool) {
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
        ) -> anyhow::Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn get(&self, _task_id: &Uuid) -> ServerResult<Task> {
            Err(ServerError::TaskNotFound(Uuid::nil()))
        }

        async fn list(&self, params: TaskListParams) -> anyhow::Result<TaskListResult> {
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
            Ok(Vec::new())
        }

        async fn find_timed_out_tasks(&self) -> ServerResult<Vec<Task>> {
            Ok(Vec::new())
        }

        async fn fail_task(&self, _task_id: &Uuid, _reason: &str) -> ServerResult<()> {
            Ok(())
        }
    }

    struct MockWorkerService {
        stale_workers: Arc<Mutex<Vec<(Uuid, Vec<Uuid>)>>>,
        worker_statuses: Arc<Mutex<HashMap<Uuid, WorkerStatus>>>,
        cleared_tasks: Arc<Mutex<Vec<Uuid>>>,
        update_status_fail_workers: Arc<Mutex<Vec<Uuid>>>,
        clear_tasks_should_fail: Arc<Mutex<bool>>,
    }

    impl MockWorkerService {
        fn new() -> Self {
            Self {
                stale_workers: Arc::new(Mutex::new(Vec::new())),
                worker_statuses: Arc::new(Mutex::new(HashMap::new())),
                cleared_tasks: Arc::new(Mutex::new(Vec::new())),
                update_status_fail_workers: Arc::new(Mutex::new(Vec::new())),
                clear_tasks_should_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn add_stale_worker(&self, worker_id: Uuid, running_tasks: Vec<Uuid>) {
            self.stale_workers
                .lock()
                .unwrap()
                .push((worker_id, running_tasks));
        }

        fn get_worker_status(&self, worker_id: &Uuid) -> Option<WorkerStatus> {
            self.worker_statuses.lock().unwrap().get(worker_id).cloned()
        }

        fn get_cleared_tasks(&self) -> Vec<Uuid> {
            self.cleared_tasks.lock().unwrap().clone()
        }

        fn set_update_status_should_fail_for_worker(&self, worker_id: Uuid) {
            self.update_status_fail_workers.lock().unwrap().push(worker_id);
        }

        fn set_clear_tasks_should_fail(&self, should_fail: bool) {
            *self.clear_tasks_should_fail.lock().unwrap() = should_fail;
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
            let status = self
                .worker_statuses
                .lock()
                .unwrap()
                .get(worker_id)
                .cloned()
                .unwrap_or(WorkerStatus::Idle);
            Ok(WorkerInfo {
                id: *worker_id,
                token: Uuid::new_v4().to_string(),
                name: "test-worker".to_string(),
                status,
                last_heartbeat: Utc::now(),
                current_tasks: vec![],
                capabilities: WorkerCapabilities::default(),
                max_concurrent: 1,
            })
        }

        async fn get_by_token(&self, _token: &str) -> ServerResult<WorkerInfo> {
            Err(ServerError::InvalidToken)
        }

        async fn list(&self) -> ServerResult<Vec<WorkerInfo>> {
            Ok(Vec::new())
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

    #[tokio::test]
    async fn test_check_workers_no_stale_workers() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let monitor = HeartbeatMonitor::new(task_service, worker_service, 60, 10);

        let result = monitor.check_workers().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_workers_with_stale_worker_no_tasks() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        worker_service.add_stale_worker(worker_id, vec![]);

        let monitor = HeartbeatMonitor::new(
            task_service.clone(),
            worker_service.clone(),
            60,
            10,
        );

        let result = monitor.check_workers().await;
        assert!(result.is_ok());

        let status = worker_service.get_worker_status(&worker_id);
        assert_eq!(status, Some(WorkerStatus::Offline));

        let requeued = task_service.get_requeued_tasks();
        assert!(requeued.is_empty());

        let cleared = worker_service.get_cleared_tasks();
        assert!(cleared.is_empty());
    }

    #[tokio::test]
    async fn test_check_workers_with_stale_worker_with_tasks() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        worker_service.add_stale_worker(worker_id, vec![task_id1, task_id2]);

        let monitor = HeartbeatMonitor::new(
            task_service.clone(),
            worker_service.clone(),
            60,
            10,
        );

        let result = monitor.check_workers().await;
        assert!(result.is_ok());

        let status = worker_service.get_worker_status(&worker_id);
        assert_eq!(status, Some(WorkerStatus::Offline));

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 2);
        assert!(requeued.contains(&task_id1));
        assert!(requeued.contains(&task_id2));

        let cleared = worker_service.get_cleared_tasks();
        assert_eq!(cleared.len(), 1);
        assert!(cleared.contains(&worker_id));
    }

    #[tokio::test]
    async fn test_check_workers_multiple_stale_workers() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id1 = Uuid::new_v4();
        let worker_id2 = Uuid::new_v4();
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        let task_id3 = Uuid::new_v4();

        worker_service.add_stale_worker(worker_id1, vec![task_id1]);
        worker_service.add_stale_worker(worker_id2, vec![task_id2, task_id3]);

        let monitor = HeartbeatMonitor::new(
            task_service.clone(),
            worker_service.clone(),
            60,
            10,
        );

        let result = monitor.check_workers().await;
        assert!(result.is_ok());

        let status1 = worker_service.get_worker_status(&worker_id1);
        let status2 = worker_service.get_worker_status(&worker_id2);
        assert_eq!(status1, Some(WorkerStatus::Offline));
        assert_eq!(status2, Some(WorkerStatus::Offline));

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 3);
        assert!(requeued.contains(&task_id1));
        assert!(requeued.contains(&task_id2));
        assert!(requeued.contains(&task_id3));

        let cleared = worker_service.get_cleared_tasks();
        assert_eq!(cleared.len(), 2);
        assert!(cleared.contains(&worker_id1));
        assert!(cleared.contains(&worker_id2));
    }

    #[tokio::test]
    async fn test_check_workers_update_status_failure() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id1 = Uuid::new_v4();
        let worker_id2 = Uuid::new_v4();
        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();

        worker_service.add_stale_worker(worker_id1, vec![task_id1]);
        worker_service.add_stale_worker(worker_id2, vec![task_id2]);

        worker_service.set_update_status_should_fail_for_worker(worker_id1);

        let monitor = HeartbeatMonitor::new(
            task_service.clone(),
            worker_service.clone(),
            60,
            10,
        );

        let result = monitor.check_workers().await;
        assert!(result.is_ok());

        let status1 = worker_service.get_worker_status(&worker_id1);
        let status2 = worker_service.get_worker_status(&worker_id2);
        assert_eq!(status1, None);
        assert_eq!(status2, Some(WorkerStatus::Offline));

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 1);
        assert!(requeued.contains(&task_id2));
    }

    #[tokio::test]
    async fn test_check_workers_requeue_failure() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        worker_service.add_stale_worker(worker_id, vec![task_id]);

        task_service.set_requeue_should_fail(true);

        let monitor = HeartbeatMonitor::new(
            task_service.clone(),
            worker_service.clone(),
            60,
            10,
        );

        let result = monitor.check_workers().await;
        assert!(result.is_ok());

        let status = worker_service.get_worker_status(&worker_id);
        assert_eq!(status, Some(WorkerStatus::Offline));

        let requeued = task_service.get_requeued_tasks();
        assert!(requeued.is_empty());

        let cleared = worker_service.get_cleared_tasks();
        assert_eq!(cleared.len(), 1);
        assert!(cleared.contains(&worker_id));
    }

    #[tokio::test]
    async fn test_check_workers_clear_tasks_failure() {
        let task_service = Arc::new(MockTaskService::new());
        let worker_service = Arc::new(MockWorkerService::new());

        let worker_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        worker_service.add_stale_worker(worker_id, vec![task_id]);

        worker_service.set_clear_tasks_should_fail(true);

        let monitor = HeartbeatMonitor::new(
            task_service.clone(),
            worker_service.clone(),
            60,
            10,
        );

        let result = monitor.check_workers().await;
        assert!(result.is_ok());

        let status = worker_service.get_worker_status(&worker_id);
        assert_eq!(status, Some(WorkerStatus::Offline));

        let requeued = task_service.get_requeued_tasks();
        assert_eq!(requeued.len(), 1);
        assert!(requeued.contains(&task_id));

        let cleared = worker_service.get_cleared_tasks();
        assert!(cleared.is_empty());
    }
}
