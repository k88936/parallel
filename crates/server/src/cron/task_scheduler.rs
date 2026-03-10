use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::service::task_service::TaskServiceTrait;
use crate::service::worker_service::WorkerServiceTrait;
use parallel_common::{TaskAssignment, WorkerInstruction};
use parallel_message_broker::MessageBrokerServer;

pub struct TaskScheduler {
    task_service: Arc<dyn TaskServiceTrait>,
    worker_service: Arc<dyn WorkerServiceTrait>,
    message_broker: MessageBrokerServer,
    check_interval_seconds: u64,
}

impl TaskScheduler {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
        message_broker: MessageBrokerServer,
        check_interval_seconds: u64,
    ) -> Self {
        Self {
            task_service,
            worker_service,
            message_broker,
            check_interval_seconds,
        }
    }

    pub async fn run(self) {
        let mut ticker = interval(Duration::from_secs(self.check_interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.assign_queued_tasks().await {
                error!("Task scheduler error: {}", e);
            }
        }
    }

    fn labels_match(worker_labels: &std::collections::HashMap<String, String>, required_labels: &std::collections::HashMap<String, String>) -> bool {
        if required_labels.is_empty() {
            return true;
        }
        
        required_labels.iter().all(|(key, value)| {
            worker_labels.get(key).map(|v| v == value).unwrap_or(false)
        })
    }

    async fn assign_queued_tasks(&self) -> anyhow::Result<()> {
        let connected_workers = self.message_broker.connected_ids();

        if connected_workers.is_empty() {
            return Ok(());
        }

        for worker_id in connected_workers {
            let has_slot = self.worker_service.has_available_slot(&worker_id).await?;
            if !has_slot {
                continue;
            }

            let worker_info = self.worker_service.get(&worker_id).await?;
            let worker_labels = &worker_info.capabilities.labels;

            let Some(task) = self.task_service.get_next_queued().await? else {
                break;
            };
            let task_id = task.id.clone();

            if !Self::labels_match(worker_labels, &task.required_labels) {
                info!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    required_labels = ?task.required_labels,
                    worker_labels = ?worker_labels,
                    "Worker labels do not match task requirements, skipping"
                );
                continue;
            }

            if let Err(e) = self.worker_service.add_task(&worker_id, &task_id).await {
                warn!(
                    worker_id = %worker_id,
                    task_id = %task.id,
                    error = %e,
                    "Failed to add task to worker"
                );
                continue;
            }

            if let Err(e) = self
                .task_service
                .set_claimed_by(&task_id, Some(worker_id))
                .await
            {
                warn!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    error = %e,
                    "Failed to set claimed_by"
                );
                continue;
            }

            let assignment = TaskAssignment {
                id: task.id,
                repo_url: task.repo_url,
                description: task.description,
                base_branch: task.base_branch,
                target_branch: task.target_branch,
                ssh_key: task.ssh_key,
                max_execution_time: task.max_execution_time,
            };
            let instruction = WorkerInstruction::AssignTask { task: assignment };
            let json = serde_json::to_string(&instruction)?;
            if !self.message_broker.send(&worker_id, json) {
                warn!(
                    worker_id = %worker_id,
                    task_id = %task_id,
                    "Failed to send task assignment (worker not connected)"
                );
                continue;
            }

            info!(
                worker_id = %worker_id,
                task_id = %task_id,
                "Task assigned to worker via scheduler"
            );
        }

        Ok(())
    }
}

pub fn spawn_task_scheduler(
    task_service: Arc<dyn TaskServiceTrait>,
    worker_service: Arc<dyn WorkerServiceTrait>,
    message_broker: MessageBrokerServer,
    check_interval_seconds: u64,
) {
    let scheduler = TaskScheduler::new(
        task_service,
        worker_service,
        message_broker,
        check_interval_seconds,
    );

    tokio::spawn(async move {
        info!(
            "Task scheduler started (interval: {}s)",
            check_interval_seconds
        );
        scheduler.run().await;
    });
}
