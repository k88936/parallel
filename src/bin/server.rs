use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,sqlx::query=warn")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./data.db?mode=rwc".to_string());
    
    let port = std::env::var("PORT")
        .map(|p| p.parse().unwrap_or(3000))
        .unwrap_or(3000);

    parallel::server::server::run_server(&database_url, port).await?;

    Ok(())
}
