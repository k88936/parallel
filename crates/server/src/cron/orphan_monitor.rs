use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::errors::ServerResult;
use crate::service::worker_service::WorkerServiceTrait;
use parallel_domain::WorkerStatus;
use crate::service::task_service::TaskServiceTrait;

pub struct OrphanMonitor {
    task_service: Arc<dyn TaskServiceTrait>,
    worker_service: Arc<dyn WorkerServiceTrait>,
    check_interval_seconds: u64,
}

impl OrphanMonitor {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
        check_interval_seconds: u64,
    ) -> Self {
        Self {
            task_service,
            worker_service,
            check_interval_seconds,
        }
    }

    pub async fn run(self) {
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.check_orphans().await {
                error!("Orphan monitor error: {}", e);
            }

            if let Err(e) = self.check_timeouts().await {
                error!("Timeout monitor error: {}", e);
            }
        }
    }

    pub async fn check_orphans(&self) -> ServerResult<()> {
        let orphaned_tasks = self.task_service.find_orphaned_tasks().await?;

        if orphaned_tasks.is_empty() {
            return Ok(());
        }

        info!(
            "Found {} orphaned tasks (no active worker)",
            orphaned_tasks.len()
        );

        for task in orphaned_tasks {
            if let Some(worker_id) = task.claimed_by {
                let worker = self.worker_service.get(&worker_id).await;

                if let Ok(worker) = worker {
                    if worker.status != WorkerStatus::Offline {
                        continue;
                    }
                }

                warn!(
                    "Re-queuing orphaned task {} (claimed by offline worker {})",
                    task.id, worker_id
                );
            } else {
                warn!("Re-queuing orphaned task {} (no worker assigned)", task.id);
            }

            if let Err(e) = self.task_service.requeue_task(&task.id).await {
                error!("Failed to re-queue orphaned task {}: {}", task.id, e);
            }
        }

        Ok(())
    }

    pub async fn check_timeouts(&self) -> ServerResult<()> {
        let timed_out_tasks = self.task_service.find_timed_out_tasks().await?;

        if timed_out_tasks.is_empty() {
            return Ok(());
        }

        info!("Found {} timed out tasks", timed_out_tasks.len());

        for task in timed_out_tasks {
            warn!(
                "Task {} exceeded max execution time ({}s), marking as Failed",
                task.id, task.max_execution_time
            );

            if let Err(e) = self
                .task_service
                .fail_task(&task.id, "Execution timeout")
                .await
            {
                error!("Failed to mark task {} as Failed: {}", task.id, e);
            }
        }

        Ok(())
    }
}

pub fn spawn_orphan_monitor(
    task_service: Arc<dyn TaskServiceTrait>,
    worker_service: Arc<dyn WorkerServiceTrait>,
    check_interval_seconds: u64,
) {
    let monitor = OrphanMonitor::new(task_service, worker_service, check_interval_seconds);

    tokio::spawn(async move {
        info!(
            "Orphan monitor started (interval: {}s)",
            check_interval_seconds
        );
        monitor.run().await;
    });
}

