use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use parallel_common::{Alert, ResourceMonitor, ReviewData, TaskStatus, WorkerEvent};

use crate::errors::ServerResult;
use crate::service::task_service::TaskServiceTrait;
use crate::service::worker_service::WorkerServiceTrait;
use crate::service::alert_service::AlertService;

pub struct EventProcessor {
    task_service: Arc<dyn TaskServiceTrait>,
    worker_service: Arc<dyn WorkerServiceTrait>,
    alert_service: AlertService,
    worker_resources: Arc<dashmap::DashMap<Uuid, ResourceMonitor>>,
}

impl EventProcessor {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
        alert_service: AlertService,
        worker_resources: Arc<dashmap::DashMap<Uuid, ResourceMonitor>>,
    ) -> Self {
        Self {
            task_service,
            worker_service,
            alert_service,
            worker_resources,
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
                    let task = self.task_service.get(&task_id).await.ok();
                    let task_title = task.map(|t| t.title).unwrap_or_else(|| task_id.to_string());
                    
                    self.task_service
                        .complete_iteration(&task_id, TaskStatus::Completed)
                        .await?;
                    
                    self.alert_service.emit(Alert::TaskCompleted {
                        task_id,
                        task_title,
                        timestamp: Utc::now(),
                    });
                }
                WorkerEvent::TaskFailed { task_id, error } => {
                    running_tasks.retain(|id| id != &task_id);
                    tracing::error!("Task {} failed: {}", task_id, error);
                    
                    let task = self.task_service.get(&task_id).await.ok();
                    let task_title = task.map(|t| t.title).unwrap_or_else(|| task_id.to_string());
                    
                    self.task_service
                        .complete_iteration(&task_id, TaskStatus::Failed)
                        .await?;
                    
                    self.alert_service.emit(Alert::TaskFailed {
                        task_id,
                        task_title,
                        error: error.clone(),
                        timestamp: Utc::now(),
                    });
                }
                WorkerEvent::TaskCancelled { task_id } => {
                    running_tasks.retain(|id| id != &task_id);
                    let task = self.task_service.get(&task_id).await.ok();
                    let task_title = task.map(|t| t.title).unwrap_or_else(|| task_id.to_string());
                    
                    self.task_service
                        .complete_iteration(&task_id, TaskStatus::Cancelled)
                        .await?;
                    
                    self.alert_service.emit(Alert::TaskCancelled {
                        task_id,
                        task_title,
                        timestamp: Utc::now(),
                    });
                }
                WorkerEvent::TaskAwaitingReview {
                    task_id,
                    messages,
                } => {
                    tracing::info!("Task {} awaiting review", task_id);

                    let task = self.task_service.get(&task_id).await.ok();
                    let task_title = task.map(|t| t.title).unwrap_or_else(|| task_id.to_string());

                    let review_data = ReviewData { messages };

                    self.task_service
                        .set_review_data(&task_id, review_data)
                        .await?;
                    
                    self.alert_service.emit(Alert::TaskReviewRequested {
                        task_id,
                        task_title,
                        worker_id: *worker_id,
                        timestamp: Utc::now(),
                    });
                }
                WorkerEvent::ResourceMonitor { resources } => {
                    tracing::debug!(
                        cpu = %resources.cpu_usage_percent,
                        mem = %resources.memory_usage_percent,
                        disk = %resources.disk_usage_percent,
                        "Worker resource monitor"
                    );
                    self.worker_resources.insert(*worker_id, resources);
                }
            }
        }

        self.worker_service
            .update_heartbeat(worker_id, running_tasks)
            .await?;

        Ok(())
    }
}

#[async_trait]
pub trait EventProcessorTrait: Send + Sync {
    async fn process_events(&self, worker_id: &Uuid, events: Vec<WorkerEvent>) -> ServerResult<()>;
}