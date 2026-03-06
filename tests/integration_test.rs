use parallel::protocol::*;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_full_task_lifecycle() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

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

    let list_response = client
        .get(&format!("{}/api/tasks", server.url))
        .send()
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_data: TaskListResponse = list_response.json().await.unwrap();
    assert!(list_data.tasks.iter().any(|t| t.id == task_id));
    assert_eq!(list_data.total, 1);

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

    let cancel_response = client
        .delete(&format!("{}/api/tasks/{}", server.url, task_id))
        .send()
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);

    let get_response = client
        .get(&format!("{}/api/tasks/{}", server.url, task_id))
        .send()
        .await
        .unwrap();

    let task: Task = get_response.json().await.unwrap();
    assert_eq!(task.status, TaskStatus::Cancelled);
}

#[tokio::test]
async fn test_worker_poll_and_events() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

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
    assert!(worker_data.current_tasks.is_empty());

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
    let low_task_id = data.task_id;

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
    let high_task_id = data.task_id;

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
    let _normal_task_id = data.task_id;

    let poll_response = client
        .post(&format!("{}/api/workers/poll", server.url))
        .json(&PollRequest { worker_id })
        .send()
        .await
        .unwrap();

    assert_eq!(poll_response.status(), StatusCode::OK);
    let poll_data: PollResponse = poll_response.json().await.unwrap();
    assert!(!poll_data.instructions.is_empty());
    
    let first_instruction = &poll_data.instructions[0];
    match first_instruction {
        WorkerInstruction::AssignTask { task } => {
            assert_eq!(task.id, high_task_id);
            assert_eq!(task.priority, TaskPriority::High);
        }
        _ => panic!("Expected AssignTask instruction"),
    }

    let events_response = client
        .post(&format!("{}/api/workers/events", server.url))
        .json(&PushEventsRequest {
            worker_id,
            events: vec![
                WorkerEvent::TaskStarted { task_id: high_task_id },
                WorkerEvent::TaskCompleted { task_id: high_task_id },
            ],
        })
        .send()
        .await
        .unwrap();

    assert_eq!(events_response.status(), StatusCode::OK);
    let events_data: PushEventsResponse = events_response.json().await.unwrap();
    assert!(events_data.acknowledged);

    let poll_response = client
        .post(&format!("{}/api/workers/poll", server.url))
        .json(&PollRequest { worker_id })
        .send()
        .await
        .unwrap();

    let poll_data: PollResponse = poll_response.json().await.unwrap();
    if let Some(WorkerInstruction::AssignTask { task }) = poll_data.instructions.first() {
        assert_eq!(task.id, low_task_id);
    }
}

#[tokio::test]
async fn test_worker_heartbeat_via_events() {
    let server = common::start_test_server().await;
    let client = reqwest::Client::new();

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

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let events_response = client
        .post(&format!("{}/api/workers/events", server.url))
        .json(&PushEventsRequest {
            worker_id,
            events: vec![WorkerEvent::Heartbeat { running_tasks: vec![] }],
        })
        .send()
        .await
        .unwrap();

    assert_eq!(events_response.status(), StatusCode::OK);

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

    let mut handles = Vec::new();
    for worker_id in worker_ids {
        let client = client.clone();
        let url = server.url.clone();
        let handle = tokio::spawn(async move {
            let resp = client
                .post(&format!("{}/api/workers/poll", url))
                .json(&PollRequest { worker_id })
                .send()
                .await
                .unwrap();
            let data: PollResponse = resp.json().await.unwrap();
            data.instructions.into_iter().filter_map(|i| {
                match i {
                    WorkerInstruction::AssignTask { task } => Some(task.id),
                    _ => None,
                }
            }).next()
        });
        handles.push(handle);
    }

    let mut claimed_task_ids: Vec<Uuid> = Vec::new();
    for handle in handles {
        if let Some(task_id) = handle.await.unwrap() {
            claimed_task_ids.push(task_id);
        }
    }
    claimed_task_ids.sort();

    assert_eq!(claimed_task_ids.len(), 3);
    task_ids.sort();
    assert_eq!(claimed_task_ids, task_ids);

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

    let resp = client
        .get(&format!("{}/api/tasks/{}", server.url, fake_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

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

    let resp = client
        .post(&format!("{}/api/workers/poll", server.url))
        .json(&PollRequest { worker_id })
        .send()
        .await
        .unwrap();
    let poll_data: PollResponse = resp.json().await.unwrap();
    assert!(!poll_data.instructions.is_empty());

    let resp = client
        .post(&format!("{}/api/workers/events", server.url))
        .json(&PushEventsRequest {
            worker_id,
            events: vec![WorkerEvent::TaskAwaitingReview {
                task_id,
                messages: vec![],
                diff: "test diff".to_string(),
            }],
        })
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

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
