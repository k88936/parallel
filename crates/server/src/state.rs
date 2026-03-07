use std::sync::Arc;

use crate::service::worker_event_service::EventProcessorTrait;
use parallel_message_broker::MessageBrokerServer;
use crate::service::project_service::ProjectServiceTrait;
use crate::service::task_service::TaskServiceTrait;
use crate::service::worker_service::WorkerServiceTrait;

#[derive(Clone)]
pub struct AppState {
    pub task_service: Arc<dyn TaskServiceTrait>,
    pub worker_service: Arc<dyn WorkerServiceTrait>,
    pub project_service: Arc<dyn ProjectServiceTrait>,
    pub event_processor: Arc<dyn EventProcessorTrait>,
    pub message_broker: MessageBrokerServer,
}

impl AppState {
    pub fn new(
        task_service: Arc<dyn TaskServiceTrait>,
        worker_service: Arc<dyn WorkerServiceTrait>,
        project_service: Arc<dyn ProjectServiceTrait>,
        event_processor: Arc<dyn EventProcessorTrait>,
        message_broker: MessageBrokerServer,
    ) -> Self {
        Self {
            task_service,
            worker_service,
            project_service,
            event_processor,
            message_broker,
        }
    }
}
