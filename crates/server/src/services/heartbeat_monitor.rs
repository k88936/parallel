use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::errors::ServerResult;
use crate::services::{TaskService, WorkerService};
use crate::state::AppState;
use parallel_protocol::WorkerStatus;

pub struct HeartbeatMonitor {
    state: AppState,
    timeout_seconds: i64,
    check_interval_seconds: u64,
}

impl HeartbeatMonitor {
    pub fn new(state: AppState, timeout_seconds: i64, check_interval_seconds: u64) -> Self {
        Self {
            state,
            timeout_seconds,
            check_interval_seconds,
        }
    }

    pub async fn run(self) {
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.check_workers().await {
                error!("Heartbeat monitor error: {}", e);
            }
        }
    }

    async fn check_workers(&self) -> ServerResult<()> {
        let worker_service = WorkerService::new(self.state.db.clone());
        let task_service = TaskService::new(self.state.db.clone());

        let stale_workers = worker_service
            .find_stale_workers(self.timeout_seconds)
            .await?;

        if stale_workers.is_empty() {
            return Ok(());
        }

        for (worker_id, running_tasks) in stale_workers {
            info!(
                "Worker {} heartbeat timeout (last heartbeat > {}s ago), marking as Offline",
                worker_id, self.timeout_seconds
            );

            if let Err(e) = worker_service
                .update_status(&worker_id, WorkerStatus::Offline)
                .await
            {
                error!("Failed to mark worker {} as Offline: {}", worker_id, e);
                continue;
            }

            if !running_tasks.is_empty() {
                warn!(
                    "Re-queuing {} tasks from offline worker {}",
                    running_tasks.len(),
                    worker_id
                );

                match task_service.requeue_tasks(&running_tasks).await {
                    Ok(count) => {
                        info!(
                            "Successfully re-queued {}/{} tasks from worker {}",
                            count,
                            running_tasks.len(),
                            worker_id
                        );
                    }
                    Err(e) => {
                        error!("Failed to re-queue tasks from worker {}: {}", worker_id, e);
                    }
                }

                if let Err(e) = worker_service.clear_tasks(&worker_id).await {
                    error!("Failed to clear tasks for worker {}: {}", worker_id, e);
                }
            }
        }

        Ok(())
    }
}

pub fn spawn_heartbeat_monitor(state: AppState, timeout_seconds: i64, check_interval_seconds: u64) {
    let monitor = HeartbeatMonitor::new(state, timeout_seconds, check_interval_seconds);

    tokio::spawn(async move {
        info!(
            "Heartbeat monitor started (timeout: {}s, interval: {}s)",
            timeout_seconds, check_interval_seconds
        );
        monitor.run().await;
    });
}
