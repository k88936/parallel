use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

use crate::protocol::{
    ClaimTaskRequest, ClaimTaskResponse, CreateTaskRequest, CreateTaskResponse,
    HeartbeatRequest, HeartbeatResponse, IterationResult, RegisterWorkerRequest,
    Task, TaskStatus, WorkerCapabilities, WorkerInfo,
};

pub struct ServerClient {
    client: Client,
    base_url: String,
}

impl ServerClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self { client, base_url }
    }

    pub async fn register(
        &self,
        name: String,
        capabilities: WorkerCapabilities,
        max_concurrent: usize,
    ) -> Result<WorkerInfo> {
        let url = format!("{}/api/workers/register", self.base_url);

        let request = RegisterWorkerRequest {
            name,
            capabilities,
            max_concurrent,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to register worker")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to register worker: status {}", response.status());
        }

        let worker_info = response
            .json::<WorkerInfo>()
            .await
            .context("Failed to parse registration response")?;

        Ok(worker_info)
    }

    pub async fn heartbeat(&self, worker_id: Uuid, current_task: Option<Uuid>) -> Result<bool> {
        let url = format!("{}/api/workers/heartbeat", self.base_url);

        let request = HeartbeatRequest {
            worker_id,
            current_task,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send heartbeat")?;

        if !response.status().is_success() {
            tracing::warn!("Heartbeat failed: status {}", response.status());
            return Ok(false);
        }

        let heartbeat_response = response
            .json::<HeartbeatResponse>()
            .await
            .context("Failed to parse heartbeat response")?;

        Ok(heartbeat_response.acknowledged)
    }

    pub async fn claim_task(&self, worker_id: Uuid) -> Result<Option<Task>> {
        let url = format!("{}/api/tasks/claim", self.base_url);

        let request = ClaimTaskRequest { worker_id };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to claim task")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to claim task: status {}", response.status());
        }

        let claim_response = response
            .json::<ClaimTaskResponse>()
            .await
            .context("Failed to parse claim response")?;

        Ok(claim_response.task)
    }

    pub async fn report_task_status(
        &self,
        task_id: Uuid,
        status: TaskStatus,
        result: Option<IterationResult>,
    ) -> Result<()> {
        let url = format!("{}/api/tasks/{}/status", self.base_url, task_id);

        let request = UpdateTaskStatusRequest { status, result };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to update task status")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to update task status: status {}",
                response.status()
            );
        }

        Ok(())
    }

    pub async fn get_task(&self, task_id: Uuid) -> Result<Task> {
        let url = format!("{}/api/tasks/{}", self.base_url, task_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get task")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get task: status {}", response.status());
        }

        let task = response
            .json::<Task>()
            .await
            .context("Failed to parse task response")?;

        Ok(task)
    }

    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Uuid> {
        let url = format!("{}/api/tasks", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to create task")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create task: status {}", response.status());
        }

        let create_response = response
            .json::<CreateTaskResponse>()
            .await
            .context("Failed to parse create task response")?;

        Ok(create_response.task_id)
    }
}

#[derive(Debug, serde::Serialize)]
struct UpdateTaskStatusRequest {
    status: TaskStatus,
    result: Option<IterationResult>,
}
