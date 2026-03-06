use sea_orm::*;
use uuid::Uuid;

use parallel_protocol::{FeedbackType, HumanFeedback, WorkerInstruction};

use crate::db::entity::workers;
use crate::errors::{ServerError, ServerResult};

pub struct Coordinator {
    db: DatabaseConnection,
}

impl Coordinator {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn queue_instruction(
        &self,
        worker_id: Uuid,
        instruction: WorkerInstruction,
    ) -> ServerResult<()> {
        let worker = workers::Entity::find_by_id(worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(worker_id))?;

        let mut pending: Vec<WorkerInstruction> =
            serde_json::from_str(&worker.pending_instructions_json)?;
        pending.push(instruction);

        let mut worker: workers::ActiveModel = worker.into();
        worker.pending_instructions_json = Set(serde_json::to_string(&pending)?);
        worker.update(&self.db).await?;

        Ok(())
    }

    pub async fn get_pending_instructions(
        &self,
        worker_id: &Uuid,
    ) -> ServerResult<Vec<WorkerInstruction>> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServerError::WorkerNotFound(*worker_id))?;

        let pending: Vec<WorkerInstruction> =
            serde_json::from_str(&worker.pending_instructions_json)?;

        if !pending.is_empty() {
            let mut worker: workers::ActiveModel = worker.into();
            worker.pending_instructions_json = Set("[]".to_string());
            worker.update(&self.db).await?;
        }

        Ok(pending)
    }

    pub async fn queue_feedback(
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

    pub async fn queue_cancellation(
        &self,
        worker_id: Uuid,
        task_id: Uuid,
        reason: String,
    ) -> ServerResult<()> {
        self.queue_instruction(worker_id, WorkerInstruction::CancelTask { task_id, reason })
            .await
    }
}
