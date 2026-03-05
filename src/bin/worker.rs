use std::path::PathBuf;
use std::env;
use tracing::{info, Level};

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy()
        )
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    info!("=== Parallel Worker Starting ===");
    info!("Initializing worker configuration");

    let work_base = PathBuf::from(
        env::var("WORKER_WORK_BASE").unwrap_or_else(|_| "./work".to_string())
    );
    let agent_path = PathBuf::from(
        env::var("WORKER_AGENT_PATH").unwrap_or_else(|_| "opencode".to_string())
    );
    let server_url = env::var("SERVER_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let worker_name = env::var("WORKER_NAME")
        .unwrap_or_else(|_| {
            format!("worker-{}", hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()))
        });
    let ssh_key_path = PathBuf::from(
        env::var("SSH_KEY_PATH").unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            format!("{}/.ssh/id_rsa", home)
        })
    );
    let max_concurrent = env::var("MAX_CONCURRENT")
        .map(|v| v.parse().unwrap_or(4))
        .unwrap_or(4);
    let ws_url = env::var("WEBSOCKET_URL")
        .unwrap_or_else(|_| "ws://localhost:3000/ws/worker".to_string());

    info!(
        work_base = %work_base.display(),
        agent_path = %agent_path.display(),
        server_url = %server_url,
        worker_name = %worker_name,
        ssh_key = %ssh_key_path.display(),
        max_concurrent = max_concurrent,
        ws_url = %ws_url,
        "Worker configuration loaded"
    );

    let mut worker = parallel::worker::Worker::new(
        work_base,
        agent_path,
        max_concurrent,
        server_url,
        ssh_key_path,
        ws_url,
    );

    info!("Registering worker with server...");
    worker.register(&worker_name).await?;

    info!("Starting worker main loop");
    worker.run().await?;

    info!("=== Parallel Worker Finished ===");

    Ok(())
}