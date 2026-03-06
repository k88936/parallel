use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

use crate::protocol::{
    CreateTaskRequest, CreateTaskResponse, WorkerCapabilities, WorkerInfo,
    Task, PollRequest, PollResponse, PushEventsRequest, PushEventsResponse,
    WorkerInstruction, WorkerEvent,
};

pub struct APIClient {
    client: Client,
    base_url: String,
}

impl APIClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
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

    pub async fn poll_instructions(&self, worker_id: Uuid) -> Result<Vec<WorkerInstruction>> {
        let url = format!("{}/api/workers/poll", self.base_url);

        let request = PollRequest { worker_id };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to poll instructions")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to poll instructions: status {}", response.status());
        }

        let poll_response = response
            .json::<PollResponse>()
            .await
            .context("Failed to parse poll response")?;

        Ok(poll_response.instructions)
    }

    pub async fn push_events(&self, worker_id: Uuid, events: Vec<WorkerEvent>) -> Result<bool> {
        let url = format!("{}/api/workers/events", self.base_url);

        let request = PushEventsRequest { worker_id, events };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to push events")?;

        if !response.status().is_success() {
            tracing::warn!("Push events failed: status {}", response.status());
            return Ok(false);
        }

        let push_response = response
            .json::<PushEventsResponse>()
            .await
            .context("Failed to parse push events response")?;

        Ok(push_response.acknowledged)
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
struct RegisterWorkerRequest {
    name: String,
    capabilities: WorkerCapabilities,
    max_concurrent: usize,
}
