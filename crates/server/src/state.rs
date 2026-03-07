use std::sync::Arc;

use crate::services::{
    CoordinatorTrait, EventProcessorTrait, ProjectServiceTrait, TaskServiceTrait,
    WorkerServiceTrait,
};
use parallel_message_broker::MessageBroker;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<dyn TaskServiceTrait>,
    pub worker_service: Arc<dyn WorkerServiceTrait>,
    pub project_service: Arc<dyn ProjectServiceTrait>,
    pub coordinator: Arc<dyn CoordinatorTrait>,
    pub event_processor: Arc<dyn EventProcessorTrait>,
    pub message_broker: MessageBroker,
}

impl AppState {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
        project_service: Arc<dyn ProjectServiceTrait>,
        coordinator: Arc<dyn CoordinatorTrait>,
        event_processor: Arc<dyn EventProcessorTrait>,
        message_broker: MessageBroker,
    ) -> Self {
        Self {
            task_service,
            worker_service,
            project_service,
            coordinator,
            event_processor,
            message_broker,
        }
    }
}
