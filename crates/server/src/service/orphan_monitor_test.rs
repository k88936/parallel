#[cfg(test)]
mod tests {
    use crate::service::orphan_monitor::OrphanMonitor;
    use crate::service::test_utils::mocks::{
        create_test_task, create_test_worker, MockTaskService, MockWorkerService,
    };
    use parallel_protocol::WorkerStatus;
    use std::sync::Arc;
    use uuid::Uuid;

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
