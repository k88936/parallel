use anyhow::{Context, Result};
use backoff::backoff::Backoff;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use parallel_protocol::{
    PollRequest, PollResponse, PushEventsRequest, PushEventsResponse, WorkerCapabilities,
    WorkerEvent, WorkerInfo, WorkerInstruction,
};

use crate::utils::default_backoff;

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
        let correlation_id = Uuid::new_v4();
        let url = format!("{}/api/workers/register", self.base_url);

        let request = RegisterWorkerRequest {
            name: name.clone(),
            capabilities,
            max_concurrent,
        };

        info!(
            correlation_id = %correlation_id,
            worker_name = %name,
            "Registering worker with server"
        );

        let response = self
            .client
            .post(&url)
            .header("x-correlation-id", correlation_id.to_string())
            .json(&request)
            .send()
            .await
            .context("Failed to register worker")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Failed to register worker: status {}, body: {}",
                status,
                body
            );
        }

        let worker_info = response
            .json::<WorkerInfo>()
            .await
            .context("Failed to parse registration response")?;

        info!(
            correlation_id = %correlation_id,
            worker_id = %worker_info.id,
            "Worker registered successfully"
        );

        Ok(worker_info)
    }

    pub async fn poll_instructions(&self, worker_id: Uuid) -> Result<Vec<WorkerInstruction>> {
        let correlation_id = Uuid::new_v4();
        let url = format!("{}/api/workers/poll", self.base_url);
        let request = PollRequest { worker_id };

        debug!(
            correlation_id = %correlation_id,
            worker_id = %worker_id,
            "Polling for instructions"
        );

        let response = self
            .client
            .post(&url)
            .header("x-correlation-id", correlation_id.to_string())
            .json(&request)
            .send()
            .await
            .context("Failed to poll instructions")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Failed to poll instructions: status {}, body: {}",
                status,
                body
            );
        }

        let poll_response = response
            .json::<PollResponse>()
            .await
            .context("Failed to parse poll response")?;

        debug!(
            correlation_id = %correlation_id,
            worker_id = %worker_id,
            instruction_count = poll_response.instructions.len(),
            "Received instructions"
        );

        Ok(poll_response.instructions)
    }

    pub async fn push_events(
        &self,
        worker_id: Uuid,
        events: Vec<WorkerEvent>,
    ) -> Result<bool> {
        let correlation_id = Uuid::new_v4();
        
        debug!(
            correlation_id = %correlation_id,
            worker_id = %worker_id,
            event_count = events.len(),
            "Pushing events to server"
        );

        let result = self
            .push_events_with_backoff(worker_id, events.clone(), correlation_id)
            .await;

        match &result {
            Ok(true) => {
                debug!(
                    correlation_id = %correlation_id,
                    worker_id = %worker_id,
                    "Events pushed successfully"
                );
            }
            Ok(false) => {
                warn!(
                    correlation_id = %correlation_id,
                    worker_id = %worker_id,
                    "Events not acknowledged by server"
                );
            }
            Err(e) => {
                warn!(
                    correlation_id = %correlation_id,
                    worker_id = %worker_id,
                    error = %e,
                    "Failed to push events after retries"
                );
            }
        }

        result
    }

    async fn push_events_with_backoff(
        &self,
        worker_id: Uuid,
        events: Vec<WorkerEvent>,
        correlation_id: Uuid,
    ) -> Result<bool> {
        let url = format!("{}/api/workers/events", self.base_url);
        let mut backoff = default_backoff();
        let mut attempt = 0;

        loop {
            attempt += 1;
            let request = PushEventsRequest {
                worker_id,
                events: events.clone(),
            };

            match self
                .client
                .post(&url)
                .header("x-correlation-id", correlation_id.to_string())
                .json(&request)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    return response
                        .json::<PushEventsResponse>()
                        .await
                        .context("Failed to parse push events response")
                        .map(|r| r.acknowledged);
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let err_msg =
                        format!("Server returned error: status {}, body: {}", status, body);

                    if status.is_client_error() {
                        warn!(
                            correlation_id = %correlation_id,
                            worker_id = %worker_id,
                            attempt = attempt,
                            status = %status,
                            "Client error, not retrying"
                        );
                        anyhow::bail!(err_msg);
                    }

                    let delay = backoff.next_backoff().unwrap_or_else(|| Duration::from_secs(60));

                    warn!(
                        correlation_id = %correlation_id,
                        worker_id = %worker_id,
                        attempt = attempt,
                        status = %status,
                        retry_after_ms = delay.as_millis(),
                        "Server error, retrying"
                    );

                    if delay == Duration::ZERO {
                        anyhow::bail!(
                            "Failed to push events after {} attempts: {}",
                            attempt,
                            err_msg
                        );
                    }

                    tokio::time::sleep(delay).await;
                }
                Err(e) => {
                    let delay = backoff.next_backoff().unwrap_or_else(|| Duration::from_secs(60));

                    warn!(
                        correlation_id = %correlation_id,
                        worker_id = %worker_id,
                        attempt = attempt,
                        error = %e,
                        retry_after_ms = delay.as_millis(),
                        "Request failed, retrying"
                    );

                    if delay == Duration::ZERO {
                        anyhow::bail!(
                            "Failed to push events after {} attempts: {}",
                            attempt,
                            e
                        );
                    }

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct RegisterWorkerRequest {
    name: String,
    capabilities: WorkerCapabilities,
    max_concurrent: usize,
}
