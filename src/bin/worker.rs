use std::env;
use std::path::PathBuf;
use tracing::{Level, info};

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    info!("=== Parallel Worker Starting ===");

    let work_base =
        PathBuf::from(env::var("WORKER_WORK_BASE").unwrap_or_else(|_| "./work".to_string()));
    let server_url = env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let worker_name = env::var("WORKER_NAME").unwrap_or_else(|_| {
        format!(
            "worker-{}",
            hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string())
        )
    });
    let max_concurrent = env::var("MAX_CONCURRENT")
        .map(|v| v.parse().unwrap_or(4))
        .unwrap_or(4);

    info!(
        work_base = %work_base.display(),
        server_url = %server_url,
        worker_name = %worker_name,
        max_concurrent = max_concurrent,
        "Worker configuration loaded"
    );

    let mut worker = parallel_worker::Worker::new(work_base, max_concurrent, server_url);

    info!("Starting worker main loop");
    worker.run().await?;

    info!("=== Parallel Worker Finished ===");
    Ok(())
}
