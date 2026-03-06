use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use uuid::Uuid;

use parallel_protocol::{
    PollRequest, PollResponse, PushEventsRequest,
    PushEventsResponse, WorkerCapabilities, WorkerEvent, WorkerInfo, WorkerInstruction,
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

        response
            .json::<WorkerInfo>()
            .await
            .context("Failed to parse registration response")
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

        response
            .json::<PollResponse>()
            .await
            .context("Failed to parse poll response")
            .map(|r| r.instructions)
    }

    pub async fn push_events(&self, worker_id: Uuid, events: Vec<WorkerEvent>) -> Result<bool> {
        self.push_events_with_retry(worker_id, events, 3).await
    }

    async fn push_events_with_retry(
        &self,
        worker_id: Uuid,
        events: Vec<WorkerEvent>,
        max_retries: u32,
    ) -> Result<bool> {
        let url = format!("{}/api/workers/events", self.base_url);
        let mut delay = Duration::from_millis(100);

        for attempt in 0..=max_retries {
            let request = PushEventsRequest {
                worker_id,
                events: events.clone(),
            };

            match self.client.post(&url).json(&request).send().await {
                Ok(response) if response.status().is_success() => {
                    return response
                        .json::<PushEventsResponse>()
                        .await
                        .context("Failed to parse push events response")
                        .map(|r| r.acknowledged);
                }
                Ok(_) | Err(_) => {
                    if attempt < max_retries {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                    } else {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(false)
    }
}

#[derive(Debug, serde::Serialize)]
struct RegisterWorkerRequest {
    name: String,
    capabilities: WorkerCapabilities,
    max_concurrent: usize,
}
