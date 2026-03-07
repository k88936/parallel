#[cfg(test)]
mod tests {
    use crate::service::heartbeat_monitor::HeartbeatMonitor;
    use crate::service::test_utils::mocks::{MockTaskService, MockWorkerService};
    use parallel_protocol::WorkerStatus;
    use std::sync::Arc;
    use uuid::Uuid;

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
