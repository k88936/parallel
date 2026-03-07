use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use parallel_message_broker::MessageBroker;
use parallel_protocol::{FeedbackType, HumanFeedback, WorkerInstruction};

use crate::errors::ServerResult;
use crate::repository::WorkerRepositoryTrait;
use crate::service::traits::CoordinatorTrait;

pub struct Coordinator<R: WorkerRepositoryTrait> {
    repository: Arc<R>,
    message_broker: MessageBroker,
}

impl<R: WorkerRepositoryTrait> Coordinator<R> {
    pub fn new(repository: Arc<R>, message_broker: MessageBroker) -> Self {
        Self { repository, message_broker }
    }
}

#[async_trait]
impl<R: WorkerRepositoryTrait + 'static> CoordinatorTrait for Coordinator<R> {
    async fn queue_instruction(
        &self,
        worker_id: Uuid,
        instruction: WorkerInstruction,
    ) -> ServerResult<()> {
        let mut pending = self.repository.get_pending_instructions(&worker_id).await?;
        pending.push(instruction.clone());

        self.repository.update_pending_instructions(&worker_id, pending).await?;

        if self.message_broker.send_instruction(&worker_id, instruction) {
            tracing::debug!(
                worker_id = %worker_id,
                "Instruction sent via WebSocket"
            );
        }

        Ok(())
    }

    async fn get_pending_instructions(
        &self,
        worker_id: &Uuid,
    ) -> ServerResult<Vec<WorkerInstruction>> {
        let pending = self.repository.get_pending_instructions(worker_id).await?;

        if !pending.is_empty() {
            self.repository.update_pending_instructions(worker_id, vec![]).await?;
        }

        Ok(pending)
    }

    async fn queue_feedback(
        &self,
        worker_id: Uuid,
        task_id: Uuid,
        feedback: HumanFeedback,
    ) -> ServerResult<()> {
        let instruction = match feedback.feedback_type {
            FeedbackType::Approve => WorkerInstruction::ApproveIteration { task_id },
            FeedbackType::RequestChanges => {
                WorkerInstruction::ProvideFeedback { task_id, feedback }
            }
            FeedbackType::Abort => WorkerInstruction::AbortTask {
                task_id,
                reason: feedback.message.clone(),
            },
        };

        self.queue_instruction(worker_id, instruction).await
    }

    async fn queue_cancellation(
        &self,
        worker_id: Uuid,
        task_id: Uuid,
        reason: String,
    ) -> ServerResult<()> {
        self.queue_instruction(worker_id, WorkerInstruction::CancelTask { task_id, reason })
            .await
    }
}
