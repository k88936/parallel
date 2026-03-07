use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use parallel_protocol::{ReviewData, TaskStatus, WorkerEvent};

use crate::errors::ServerResult;
use crate::service::traits::{EventProcessorTrait, TaskServiceTrait, WorkerServiceTrait};

pub struct EventProcessor {
    task_service: Arc<dyn TaskServiceTrait>,
    worker_service: Arc<dyn WorkerServiceTrait>,
}

impl EventProcessor {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
    ) -> Self {
        Self {
            task_service,
            worker_service,
        }
    }
}

#[async_trait]
impl EventProcessorTrait for EventProcessor {
    async fn process_events(&self, worker_id: &Uuid, events: Vec<WorkerEvent>) -> ServerResult<()> {
        let mut running_tasks = self.worker_service.get_running_tasks(worker_id).await?;

        for event in events {
            match event {
                WorkerEvent::Heartbeat {
                    running_tasks: tasks,
                } => {
                    running_tasks = tasks;
                }
                WorkerEvent::TaskStarted { task_id } => {
                    if !running_tasks.contains(&task_id) {
                        running_tasks.push(task_id);
                    }
                    self.task_service
                        .update_status(&task_id, TaskStatus::InProgress)
                        .await?;
                }
                WorkerEvent::TaskProgress {
                    task_id,
                    message: _,
                } => {
                    tracing::info!("Task {} progress", task_id);
                }
                WorkerEvent::TaskCompleted { task_id } => {
                    running_tasks.retain(|id| id != &task_id);
                    self.task_service
                        .complete_iteration(&task_id, TaskStatus::Completed)
                        .await?;
                }
                WorkerEvent::TaskFailed { task_id, error } => {
                    running_tasks.retain(|id| id != &task_id);
                    tracing::error!("Task {} failed: {}", task_id, error);
                    self.task_service
                        .complete_iteration(&task_id, TaskStatus::Failed)
                        .await?;
                }
                WorkerEvent::TaskCancelled { task_id } => {
                    running_tasks.retain(|id| id != &task_id);
                    self.task_service
                        .complete_iteration(&task_id, TaskStatus::Cancelled)
                        .await?;
                }
                WorkerEvent::TaskAwaitingReview {
                    task_id,
                    messages,
                    diff,
                } => {
                    tracing::info!("Task {} awaiting review", task_id);

                    let review_data = ReviewData { messages, diff };

                    self.task_service
                        .set_review_data(&task_id, review_data)
                        .await?;
                }
            }
        }

        self.worker_service
            .update_heartbeat(worker_id, running_tasks)
            .await?;

        Ok(())
    }
}
