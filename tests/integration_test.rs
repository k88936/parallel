use parallel::protocol::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_full_task_lifecycle() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    // 1. Create a task
    let create_response = client
        .post(&format!("{}/api/tasks", server.url))
        .json(&json!({
            "repo_url": "git@github.com:test/repo.git",
            "description": "Add integration tests",
            "priority": "high"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_data: CreateTaskResponse = create_response.json().await.unwrap();
    let task_id = create_data.task_id;

    // 2. List tasks and verify the task exists
    let list_response = client
        .get(&format!("{}/api/tasks", server.url))
        .send()
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_data: TaskListResponse = list_response.json().await.unwrap();
    assert!(list_data.tasks.iter().any(|t| t.id == task_id));
    assert_eq!(list_data.total, 1);

    // 3. Get the specific task
    let get_response = client
        .get(&format!("{}/api/tasks/{}", server.url, task_id))
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);
    let task: Task = get_response.json().await.unwrap();
    assert_eq!(task.id, task_id);
    assert_eq!(task.status, TaskStatus::Queued);
    assert_eq!(task.description, "Add integration tests");
    assert_eq!(task.priority, TaskPriority::High);
    assert!(task.target_branch.starts_with("task/"));
    assert_eq!(task.base_branch, "main");
    assert!(task.claimed_by.is_none());
    assert_eq!(task.current_iteration, 0);

    // 4. Cancel the task
    let cancel_response = client
        .delete(&format!("{}/api/tasks/{}", server.url, task_id))
        .send()
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);

    // 5. Verify the task is cancelled
    let get_response = client
        .get(&format!("{}/api/tasks/{}", server.url, task_id))
        .send()
        .await
        .unwrap();

    let task: Task = get_response.json().await.unwrap();
    assert_eq!(task.status, TaskStatus::Cancelled);
}

#[tokio::test]
async fn test_worker_registration_and_task_claiming() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    // 1. Register a worker
    let register_response = client
        .post(&format!("{}/api/workers/register", server.url))
        .json(&json!({
            "name": "test-worker-01",
            "capabilities": {
                "has_git": true,
                "has_opencode": true,
                "supported_languages": ["rust", "python"]
            },
            "max_concurrent": 4
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::OK);
    let worker_data: WorkerInfo = register_response.json().await.unwrap();
    let worker_id = worker_data.id;
    assert_eq!(worker_data.name, "test-worker-01");
    assert_eq!(worker_data.status, WorkerStatus::Idle);

    // 2. Create multiple tasks with different priorities
    let mut task_ids = Vec::new();
    
    // Low priority task
    let resp = client
        .post(&format!("{}/api/tasks", server.url))
        .json(&json!({
            "repo_url": "git@github.com:test/repo1.git",
            "description": "Low priority task",
            "priority": "low"
        }))
        .send()
        .await
        .unwrap();
    let data: CreateTaskResponse = resp.json().await.unwrap();
    task_ids.push(data.task_id);

    // High priority task
    let resp = client
        .post(&format!("{}/api/tasks", server.url))
        .json(&json!({
            "repo_url": "git@github.com:test/repo2.git",
            "description": "High priority task",
            "priority": "high"
        }))
        .send()
        .await
        .unwrap();
    let data: CreateTaskResponse = resp.json().await.unwrap();
    task_ids.push(data.task_id);

    // Normal priority task
    let resp = client
        .post(&format!("{}/api/tasks", server.url))
        .json(&json!({
            "repo_url": "git@github.com:test/repo3.git",
            "description": "Normal priority task"
        }))
        .send()
        .await
        .unwrap();
    let data: CreateTaskResponse = resp.json().await.unwrap();
    task_ids.push(data.task_id);

    // 3. Claim a task - should get the high priority one
    let claim_response = client
        .post(&format!("{}/api/tasks/claim", server.url))
        .json(&ClaimTaskRequest { worker_id })
        .send()
        .await
        .unwrap();

    assert_eq!(claim_response.status(), StatusCode::OK);
    let claim_data: ClaimTaskResponse = claim_response.json().await.unwrap();
    let claimed_task = claim_data.task.expect("Should claim a task");
    
    assert_eq!(claimed_task.priority, TaskPriority::High);
    assert_eq!(claimed_task.status, TaskStatus::Claimed);
    assert_eq!(claimed_task.claimed_by, Some(worker_id));
    assert_eq!(claimed_task.current_iteration, 1);
    assert_eq!(claimed_task.iterations.len(), 1);
    assert!(claimed_task.iterations[0].started_at <= chrono::Utc::now());
    assert!(claimed_task.iterations[0].completed_at.is_none());

    // 4. Claim another task - should get normal priority
    let claim_response = client
        .post(&format!("{}/api/tasks/claim", server.url))
        .json(&ClaimTaskRequest { worker_id })
        .send()
        .await
        .unwrap();

    let claim_data: ClaimTaskResponse = claim_response.json().await.unwrap();
    let claimed_task = claim_data.task.expect("Should claim a task");
    assert_eq!(claimed_task.priority, TaskPriority::Normal);

    // 5. Claim another task - should get low priority
    let claim_response = client
        .post(&format!("{}/api/tasks/claim", server.url))
        .json(&ClaimTaskRequest { worker_id })
        .send()
        .await
        .unwrap();

    let claim_data: ClaimTaskResponse = claim_response.json().await.unwrap();
    let claimed_task = claim_data.task.expect("Should claim a task");
    assert_eq!(claimed_task.priority, TaskPriority::Low);

    // 6. Try to claim when no tasks available
    let claim_response = client
        .post(&format!("{}/api/tasks/claim", server.url))
        .json(&ClaimTaskRequest { worker_id })
        .send()
        .await
        .unwrap();

    let claim_data: ClaimTaskResponse = claim_response.json().await.unwrap();
    assert!(claim_data.task.is_none());
}

#[tokio::test]
async fn test_list_tasks_with_filters() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    // Create tasks
    for i in 0..5 {
        client
            .post(&format!("{}/api/tasks", server.url))
            .json(&json!({
                "repo_url": format!("git@github.com:test/repo{}.git", i),
                "description": format!("Task {}", i)
            }))
            .send()
            .await
            .unwrap();
    }

    // List all tasks
    let resp = client
        .get(&format!("{}/api/tasks", server.url))
        .send()
        .await
        .unwrap();
    let data: TaskListResponse = resp.json().await.unwrap();
    assert_eq!(data.tasks.len(), 5);
    assert_eq!(data.total, 5);

    // List with limit
    let resp = client
        .get(&format!("{}/api/tasks?limit=2", server.url))
        .send()
        .await
        .unwrap();
    let data: TaskListResponse = resp.json().await.unwrap();
    assert_eq!(data.tasks.len(), 2);
    assert_eq!(data.total, 5); // Total still 5

    // List with offset
    let resp = client
        .get(&format!("{}/api/tasks?offset=2&limit=2", server.url))
        .send()
        .await
        .unwrap();
    let data: TaskListResponse = resp.json().await.unwrap();
    assert_eq!(data.tasks.len(), 2);
    assert_eq!(data.total, 5);

    // Register worker and claim a task
    let resp = client
        .post(&format!("{}/api/workers/register", server.url))
        .json(&json!({
            "name": "test-worker",
            "capabilities": {
                "has_git": true,
                "has_opencode": true,
                "supported_languages": []
            },
            "max_concurrent": 1
        }))
        .send()
        .await
        .unwrap();
    let worker_data: WorkerInfo = resp.json().await.unwrap();

    client
        .post(&format!("{}/api/tasks/claim", server.url))
        .json(&ClaimTaskRequest { worker_id: worker_data.id })
        .send()
        .await
        .unwrap();

    // List only queued tasks
    let resp = client
        .get(&format!("{}/api/tasks?status=queued", server.url))
        .send()
        .await
        .unwrap();
    let data: TaskListResponse = resp.json().await.unwrap();
    assert_eq!(data.tasks.len(), 4);
    assert_eq!(data.total, 4);

    // List only claimed tasks
    let resp = client
        .get(&format!("{}/api/tasks?status=claimed", server.url))
        .send()
        .await
        .unwrap();
    let data: TaskListResponse = resp.json().await.unwrap();
    assert_eq!(data.tasks.len(), 1);
    assert_eq!(data.total, 1);
}

#[tokio::test]
async fn test_worker_heartbeat() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    // Register worker
    let resp = client
        .post(&format!("{}/api/workers/register", server.url))
        .json(&json!({
            "name": "test-worker",
            "capabilities": {
                "has_git": true,
                "has_opencode": true,
                "supported_languages": []
            },
            "max_concurrent": 1
        }))
        .send()
        .await
        .unwrap();
    let worker_data: WorkerInfo = resp.json().await.unwrap();
    let worker_id = worker_data.id;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Send heartbeat
    let heartbeat_response = client
        .post(&format!("{}/api/workers/heartbeat", server.url))
        .json(&json!({ "worker_id": worker_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    // Verify worker updated
    let list_response = client
        .get(&format!("{}/api/workers", server.url))
        .send()
        .await
        .unwrap();

    let workers: Vec<WorkerInfo> = list_response.json().await.unwrap();
    let worker = workers.iter().find(|w| w.id == worker_id).unwrap();
    assert!(worker.last_heartbeat > worker_data.last_heartbeat);
}

#[tokio::test]
async fn test_concurrent_task_claiming() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    // Create tasks
    let mut task_ids = Vec::new();
    for i in 0..3 {
        let resp = client
            .post(&format!("{}/api/tasks", server.url))
            .json(&json!({
                "repo_url": format!("git@github.com:test/repo{}.git", i),
                "description": format!("Task {}", i)
            }))
            .send()
            .await
            .unwrap();
        let data: CreateTaskResponse = resp.json().await.unwrap();
        task_ids.push(data.task_id);
    }

    // Register multiple workers
    let mut worker_ids = Vec::new();
    for i in 0..3 {
        let resp = client
            .post(&format!("{}/api/workers/register", server.url))
            .json(&json!({
                "name": format!("worker-{}", i),
                "capabilities": {
                    "has_git": true,
                    "has_opencode": true,
                    "supported_languages": []
                },
                "max_concurrent": 1
            }))
            .send()
            .await
            .unwrap();
        let data: WorkerInfo = resp.json().await.unwrap();
        worker_ids.push(data.id);
    }

    // Claim tasks concurrently
    let mut handles = Vec::new();
    for worker_id in worker_ids {
        let client = client.clone();
        let url = server.url.clone();
        let handle = tokio::spawn(async move {
            let resp = client
                .post(&format!("{}/api/tasks/claim", url))
                .json(&ClaimTaskRequest { worker_id })
                .send()
                .await
                .unwrap();
            let data: ClaimTaskResponse = resp.json().await.unwrap();
            data.task.map(|t| t.id)
        });
        handles.push(handle);
    }

    // Collect results
    let mut claimed_task_ids: Vec<Uuid> = Vec::new();
    for handle in handles {
        if let Some(task_id) = handle.await.unwrap() {
            claimed_task_ids.push(task_id);
        }
    }
    claimed_task_ids.sort();

    // Verify all tasks were claimed exactly once
    assert_eq!(claimed_task_ids.len(), 3);
    task_ids.sort();
    assert_eq!(claimed_task_ids, task_ids);

    // Verify no double-claiming
    let resp = client
        .get(&format!("{}/api/tasks?status=queued", server.url))
        .send()
        .await
        .unwrap();
    let data: TaskListResponse = resp.json().await.unwrap();
    assert_eq!(data.total, 0);
}

#[tokio::test]
async fn test_task_not_found() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    let fake_id = Uuid::new_v4();

    // Get non-existent task
    let resp = client
        .get(&format!("{}/api/tasks/{}", server.url, fake_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Cancel non-existent task
    let resp = client
        .delete(&format!("{}/api/tasks/{}", server.url, fake_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_task_with_custom_branches() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(&format!("{}/api/tasks", server.url))
        .json(&json!({
            "repo_url": "git@github.com:test/repo.git",
            "description": "Custom branch task",
            "base_branch": "develop",
            "target_branch": "feature/my-feature"
        }))
        .send()
        .await
        .unwrap();

    let data: CreateTaskResponse = resp.json().await.unwrap();

    let resp = client
        .get(&format!("{}/api/tasks/{}", server.url, data.task_id))
        .send()
        .await
        .unwrap();

    let task: Task = resp.json().await.unwrap();
    assert_eq!(task.base_branch, "develop");
    assert_eq!(task.target_branch, "feature/my-feature");
}

#[tokio::test]
async fn test_submit_feedback() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

    // Create task
    let resp = client
        .post(&format!("{}/api/tasks", server.url))
        .json(&json!({
            "repo_url": "git@github.com:test/repo.git",
            "description": "Test task"
        }))
        .send()
        .await
        .unwrap();
    let data: CreateTaskResponse = resp.json().await.unwrap();
    let task_id = data.task_id;

    // Submit feedback
    let resp = client
        .post(&format!("{}/api/tasks/{}/feedback", server.url, task_id))
        .json(&json!({
            "feedback_type": "request_changes",
            "message": "Please improve the implementation"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
