use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::Mutex;
use std::time::Duration;
use parallel_server::run_server;

static NEXT_PORT: AtomicU16 = AtomicU16::new(3001);
static SERVER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub struct TestServer {
    pub url: String,
}

pub async fn start_test_server() -> TestServer {
    let _lock = SERVER_LOCK.lock().await;
    
    let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
    let database_url = "sqlite::memory:?cache=shared".to_string();
    
    // Start a server in the background
    let db_url = database_url.clone();
    tokio::spawn(async move {
        run_server(&db_url, port)
            .await
            .expect("Server failed to start");
    });

    // Wait for the server to be ready
    let url = format!("http://localhost:{}", port);
    let client = reqwest::Client::new();
    
    let mut retries = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        if let Ok(resp) = client
            .get(&format!("{}/api/tasks", url))
            .send()
            .await
        {
            if resp.status().is_success() {
                break;
            }
        }
        
        retries += 1;
        if retries > 20 {
            panic!("Server failed to start within 1 second");
        }
    }

    TestServer { url }
}
