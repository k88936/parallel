use anyhow::Result;
use backoff::{ExponentialBackoff, backoff::Backoff};
use std::time::Duration;
use tracing::{debug, warn};

pub fn default_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        initial_interval: Duration::from_millis(100),
        max_interval: Duration::from_secs(60),
        multiplier: 2.0,
        max_elapsed_time: Some(Duration::from_secs(300)),
        ..Default::default()
    }
}

pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut backoff = default_backoff();
    let mut attempt = 0;

    loop {
        attempt += 1;
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                let delay = backoff.next_backoff().unwrap_or_else(|| Duration::from_secs(60));
                
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    error = %err,
                    retry_after_ms = delay.as_millis(),
                    "Operation failed, retrying"
                );

                if delay == Duration::ZERO {
                    anyhow::bail!(
                        "Operation '{}' failed after {} attempts: {}",
                        operation_name,
                        attempt,
                        err
                    );
                }

                tokio::time::sleep(delay).await;
            }
        }
    }
}

pub fn make_correlation_id(task_id: Option<uuid::Uuid>) -> uuid::Uuid {
    task_id.unwrap_or_else(uuid::Uuid::new_v4)
}
