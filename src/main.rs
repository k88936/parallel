mod worker;

use std::path::PathBuf;
use tracing::{info, Level};
use worker::{Task, Worker};

fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(Level::DEBUG.into())
                .from_env_lossy()
        )
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    info!("=== Parallel Worker Starting ===");
    info!("Initializing worker configuration");

    let work_base = PathBuf::from("./work");
    let agent_path = PathBuf::from("opencode");
    let max_concurrent = 4;

    info!(
        work_base = %work_base.display(),
        agent_path = %agent_path.display(),
        max_concurrent = max_concurrent,
        "Worker configuration loaded"
    );

    let worker = Worker::new(work_base, agent_path, max_concurrent);

    let task = Task {
        id: "task-001".to_string(),
        repo_url: "git@github.com:k88936/test.git".to_string(),
        description: "write hello world to README.md".to_string(),
        branch_name: "task/task-001".to_string(),
        ssh_key_path: PathBuf::from("~/.ssh/id_rsa"),
    };

    info!(
        task_id = %task.id,
        repo_url = %task.repo_url,
        description = %task.description,
        branch = %task.branch_name,
        "Submitting task for execution"
    );

    let result = worker.execute(task).await;

    info!(
        success = result.success,
        task_id = %result.task_id,
        branch = %result.branch_name,
        error = ?result.error,
        "Task execution completed"
    );

    info!("=== Parallel Worker Finished ===");

    Ok(())
}
