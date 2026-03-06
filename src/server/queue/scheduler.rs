use anyhow::Result;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;

use crate::protocol::{Task, TaskStatus, TaskPriority, WorkerInstruction, WorkerEvent, HumanFeedback, FeedbackType, ReviewData};
use crate::server::db::entity::{tasks, workers};

pub struct TaskScheduler {
    db: DatabaseConnection,
}

impl TaskScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create_task(
        &self,
        repo_url: String,
        description: String,
        base_branch: String,
        target_branch: String,
        priority: TaskPriority,
    ) -> Result<Uuid> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let task = tasks::ActiveModel {
            id: Set(task_id),
            repo_url: Set(repo_url),
            description: Set(description),
            base_branch: Set(base_branch),
            target_branch: Set(target_branch),
            status: Set(TaskStatus::Queued.as_str().to_string()),
            priority: Set(priority.as_i32()),
            created_at: Set(now),
            updated_at: Set(now),
            claimed_by: Set(None),
            review_data_json: Set(None),
        };

        tasks::Entity::insert(task)
            .exec(&self.db)
            .await?;

        Ok(task_id)
    }

    pub async fn get_task(&self, task_id: &Uuid) -> Result<Task> {
        let task_model = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        Ok(Task {
            id: task_model.id,
            repo_url: task_model.repo_url,
            description: task_model.description,
            base_branch: task_model.base_branch,
            target_branch: task_model.target_branch,
            status: TaskStatus::from_str(&task_model.status)
                .unwrap_or(TaskStatus::Created),
            priority: TaskPriority::from_i32(task_model.priority)
                .unwrap_or(TaskPriority::Normal),
            created_at: task_model.created_at,
            updated_at: task_model.updated_at,
            claimed_by: task_model.claimed_by,
        })
    }

    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<Task>> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        query = query
            .order_by_desc(tasks::Column::CreatedAt)
            .limit(limit)
            .offset(offset);

        let tasks = query.all(&self.db).await?;

        let mut result = Vec::new();
        for task_model in tasks {
            let task = self.get_task(&task_model.id).await?;
            result.push(task);
        }

        Ok(result)
    }

    pub async fn count_tasks(&self, status: Option<TaskStatus>) -> Result<u64> {
        let mut query = tasks::Entity::find();

        if let Some(s) = status {
            query = query.filter(tasks::Column::Status.eq(s.as_str()));
        }

        let count = query.count(&self.db).await?;

        Ok(count)
    }

    pub async fn cancel_task(&self, task_id: &Uuid) -> Result<()> {
        let task = self.get_task(task_id).await?;
        
        if let Some(worker_id) = task.claimed_by {
            self.queue_instruction(worker_id, WorkerInstruction::CancelTask {
                task_id: *task_id,
                reason: "Cancelled by user".to_string(),
            }).await?;
        }
        
        self.update_task_status(task_id, TaskStatus::Cancelled).await
    }

    pub async fn update_task_status(&self, task_id: &Uuid, status: TaskStatus) -> Result<()> {
        let now = Utc::now();

        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let mut task: tasks::ActiveModel = task.into();
        task.status = Set(status.as_str().to_string());
        task.updated_at = Set(now);
        task.update(&self.db).await?;

        Ok(())
    }

    pub async fn complete_iteration(
        &self,
        task_id: &Uuid,
        status: TaskStatus
    ) -> Result<()> {
        let now = Utc::now();

        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let mut task_update: tasks::ActiveModel = task.into();
        task_update.status = Set(status.as_str().to_string());
        task_update.updated_at = Set(now);
        
        if status == TaskStatus::Completed || status == TaskStatus::Cancelled {
            task_update.claimed_by = Set(None);
        }
        
        task_update.update(&self.db).await?;

        Ok(())
    }

    pub async fn poll_instructions(&self, worker_id: &Uuid) -> Result<Vec<WorkerInstruction>> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Worker not found"))?;

        let pending: Vec<WorkerInstruction> = serde_json::from_str(&worker.pending_instructions_json)
            .unwrap_or_default();

        if !pending.is_empty() {
            let mut worker_update: workers::ActiveModel = worker.into();
            worker_update.pending_instructions_json = Set("[]".to_string());
            worker_update.update(&self.db).await?;
        }

        if pending.is_empty() {
            let available_slot = self.get_available_slot(worker_id).await?;
            if let Some(task) = self.get_next_queued_task().await? {
                if available_slot {
                    self.assign_task_to_worker(&task.id, worker_id).await?;
                    return Ok(vec![WorkerInstruction::AssignTask { task }]);
                }
            }
        }

        Ok(pending)
    }

    pub async fn process_events(&self, worker_id: &Uuid, events: Vec<WorkerEvent>) -> Result<()> {
        let now = Utc::now();
        
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Worker not found"))?;

        let mut running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)
            .unwrap_or_default();

        for event in events {
            match event {
                WorkerEvent::Heartbeat { running_tasks: tasks } => {
                    running_tasks = tasks;
                }
                WorkerEvent::TaskStarted { task_id } => {
                    if !running_tasks.contains(&task_id) {
                        running_tasks.push(task_id);
                    }
                    self.update_task_status(&task_id, TaskStatus::InProgress).await?;
                }
                WorkerEvent::TaskProgress { task_id, message: _ } => {
                    tracing::info!("Task {} progress", task_id);
                }
                WorkerEvent::TaskCompleted { task_id } => {
                    running_tasks.retain(|id| id != &task_id);
                    self.complete_iteration(&task_id, TaskStatus::Completed).await?;
                }
                WorkerEvent::TaskFailed { task_id, error } => {
                    running_tasks.retain(|id| id != &task_id);
                    tracing::error!("Task {} failed: {}", task_id, error);
                    self.complete_iteration(&task_id, TaskStatus::Failed).await?;
                }
                WorkerEvent::TaskCancelled { task_id } => {
                    running_tasks.retain(|id| id != &task_id);
                    self.complete_iteration(&task_id, TaskStatus::Cancelled).await?;
                }
                WorkerEvent::TaskAwaitingReview { task_id, messages, diff } => {
                    tracing::info!("Task {} awaiting review", task_id);
                    
                    let review_data = ReviewData {
                        messages,
                        diff,
                    };
                    let review_data_json = serde_json::to_string(&review_data)?;
                    
                    let task = tasks::Entity::find_by_id(task_id)
                        .one(&self.db)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
                    
                    let mut task_update: tasks::ActiveModel = task.into();
                    task_update.status = Set(TaskStatus::AwaitingReview.as_str().to_string());
                    task_update.review_data_json = Set(Some(review_data_json));
                    task_update.updated_at = Set(now);
                    task_update.update(&self.db).await?;
                }
            }
        }

        let mut worker_update: workers::ActiveModel = worker.into();
        worker_update.last_heartbeat = Set(now);
        worker_update.current_tasks_json = Set(serde_json::to_string(&running_tasks)?);
        worker_update.status = Set(if running_tasks.is_empty() {
            "idle".to_string()
        } else {
            "busy".to_string()
        });
        worker_update.update(&self.db).await?;

        Ok(())
    }

    pub async fn submit_feedback(&self, task_id: &Uuid, feedback: HumanFeedback) -> Result<()> {
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        
        let worker_id = task.claimed_by
            .ok_or_else(|| anyhow::anyhow!("Task not claimed by any worker"))?;
        
        let instruction = match feedback.feedback_type {
            FeedbackType::Approve => WorkerInstruction::ApproveIteration { task_id: *task_id },
            FeedbackType::RequestChanges => WorkerInstruction::ProvideFeedback {
                task_id: *task_id,
                feedback,
            },
            FeedbackType::Abort => WorkerInstruction::AbortTask {
                task_id: *task_id,
                reason: feedback.message.clone(),
            },
        };
        
        self.queue_instruction(worker_id, instruction).await
    }

    pub async fn get_review_data(&self, task_id: &Uuid) -> Result<Option<ReviewData>> {
        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        
        match task.review_data_json {
            Some(json) => {
                let review_data: ReviewData = serde_json::from_str(&json)?;
                Ok(Some(review_data))
            }
            None => Ok(None),
        }
    }

    async fn queue_instruction(&self, worker_id: Uuid, instruction: WorkerInstruction) -> Result<()> {
        let worker = workers::Entity::find_by_id(worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Worker not found"))?;

        let mut pending: Vec<WorkerInstruction> = serde_json::from_str(&worker.pending_instructions_json)
            .unwrap_or_default();
        pending.push(instruction);

        let mut worker_update: workers::ActiveModel = worker.into();
        worker_update.pending_instructions_json = Set(serde_json::to_string(&pending)?);
        worker_update.update(&self.db).await?;

        Ok(())
    }

    async fn get_available_slot(&self, worker_id: &Uuid) -> Result<bool> {
        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Worker not found"))?;

        let running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)
            .unwrap_or_default();

        Ok(running_tasks.len() < worker.max_concurrent as usize)
    }

    async fn get_next_queued_task(&self) -> Result<Option<Task>> {
        let task = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Queued.as_str()))
            .order_by_desc(tasks::Column::Priority)
            .order_by_asc(tasks::Column::CreatedAt)
            .one(&self.db)
            .await?;

        Ok(task.map(|t| Task {
            id: t.id,
            repo_url: t.repo_url,
            description: t.description,
            base_branch: t.base_branch,
            target_branch: t.target_branch,
            status: TaskStatus::from_str(&t.status).unwrap_or(TaskStatus::Created),
            priority: TaskPriority::from_i32(t.priority).unwrap_or(TaskPriority::Normal),
            created_at: t.created_at,
            updated_at: t.updated_at,
            claimed_by: t.claimed_by,
        }))
    }

    async fn assign_task_to_worker(&self, task_id: &Uuid, worker_id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let task = tasks::Entity::find_by_id(*task_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        let mut task_update: tasks::ActiveModel = task.into();
        task_update.status = Set(TaskStatus::Claimed.as_str().to_string());
        task_update.claimed_by = Set(Some(*worker_id));
        task_update.updated_at = Set(now);
        task_update.update(&self.db).await?;

        let worker = workers::Entity::find_by_id(*worker_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Worker not found"))?;

        let mut running_tasks: Vec<Uuid> = serde_json::from_str(&worker.current_tasks_json)
            .unwrap_or_default();
        running_tasks.push(*task_id);

        let mut worker_update: workers::ActiveModel = worker.into();
        worker_update.current_tasks_json = Set(serde_json::to_string(&running_tasks)?);
        worker_update.status = Set("busy".to_string());
        worker_update.update(&self.db).await?;

        Ok(())
    }
}
