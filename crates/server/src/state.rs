use std::sync::Arc;

use crate::services::{
    CoordinatorTrait, EventProcessorTrait, TaskServiceTrait, WorkerServiceTrait,
};

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<dyn TaskServiceTrait>,
    pub worker_service: Arc<dyn WorkerServiceTrait>,
    pub coordinator: Arc<dyn CoordinatorTrait>,
    pub event_processor: Arc<dyn EventProcessorTrait>,
}

impl AppState {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
        coordinator: Arc<dyn CoordinatorTrait>,
        event_processor: Arc<dyn EventProcessorTrait>,
    ) -> Self {
        Self {
            task_service,
            worker_service,
            coordinator,
            event_processor,
        }
    }
}
