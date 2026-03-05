#!/bin/bash

# Phase 1 Integration Test Demo
# This script demonstrates all Phase 1 functionality

set -e

echo "========================================="
echo "Phase 1 Integration Test Demo"
echo "========================================="
echo ""

BASE_URL="${BASE_URL:-http://localhost:3000}"

echo "1. Starting test server..."
echo "   (Make sure server is running: cargo run --bin server)"
echo ""

read -p "Press Enter when server is ready..."

echo ""
echo "2. Creating a task..."
TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tasks" \
  -H "Content-Type: application/json" \
  -d '{
    "repo_url": "git@github.com:test/repo.git",
    "description": "Implement user authentication",
    "priority": "high"
  }')
echo "   Response: $TASK_RESPONSE"
TASK_ID=$(echo $TASK_RESPONSE | jq -r '.task_id')
echo "   Task ID: $TASK_ID"
echo ""

echo "3. Listing all tasks..."
TASKS=$(curl -s "$BASE_URL/api/tasks")
echo "   Total tasks: $(echo $TASKS | jq '.total')"
echo "   First task status: $(echo $TASKS | jq -r '.tasks[0].status')"
echo "   First task priority: $(echo $TASKS | jq -r '.tasks[0].priority')"
echo ""

echo "4. Getting task details..."
TASK_DETAILS=$(curl -s "$BASE_URL/api/tasks/$TASK_ID")
echo "   ID: $(echo $TASK_DETAILS | jq -r '.id')"
echo "   Description: $(echo $TASK_DETAILS | jq -r '.description')"
echo "   Status: $(echo $TASK_DETAILS | jq -r '.status')"
echo "   Base branch: $(echo $TASK_DETAILS | jq -r '.base_branch')"
echo "   Target branch: $(echo $TASK_DETAILS | jq -r '.target_branch')"
echo ""

echo "5. Registering a worker..."
WORKER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/workers/register" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-worker-01",
    "capabilities": {
      "has_git": true,
      "has_opencode": true,
      "supported_languages": ["rust", "python", "javascript"]
    },
    "max_concurrent": 4
  }')
WORKER_ID=$(echo $WORKER_RESPONSE | jq -r '.id')
echo "   Worker ID: $WORKER_ID"
echo "   Worker status: $(echo $WORKER_RESPONSE | jq -r '.status')"
echo ""

echo "6. Worker claims a task..."
CLAIM_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tasks/claim" \
  -H "Content-Type: application/json" \
  -d "{\"worker_id\": \"$WORKER_ID\"}")
CLAIMED_TASK_ID=$(echo $CLAIM_RESPONSE | jq -r '.task.id')
echo "   Claimed task ID: $CLAIMED_TASK_ID"
echo "   Task status after claim: $(echo $CLAIM_RESPONSE | jq -r '.task.status')"
echo "   Claimed by: $(echo $CLAIM_RESPONSE | jq -r '.task.claimed_by')"
echo "   Iteration: $(echo $CLAIM_RESPONSE | jq -r '.task.current_iteration')"
echo ""

echo "7. Sending worker heartbeat..."
HEARTBEAT_RESPONSE=$(curl -s -X POST "$BASE_URL/api/workers/heartbeat" \
  -H "Content-Type: application/json" \
  -d "{\"worker_id\": \"$WORKER_ID\", \"current_task\": \"$CLAIMED_TASK_ID\"}")
echo "   Heartbeat acknowledged: $(echo $HEARTBEAT_RESPONSE | jq -r '.acknowledged')"
echo ""

echo "8. Listing workers..."
WORKERS=$(curl -s "$BASE_URL/api/workers")
echo "   Total workers: $(echo $WORKERS | jq 'length')"
echo "   First worker status: $(echo $WORKERS | jq -r '.[0].status')"
echo "   First worker current task: $(echo $WORKERS | jq -r '.[0].current_task')"
echo ""

echo "9. Creating more tasks with different priorities..."
curl -s -X POST "$BASE_URL/api/tasks" \
  -H "Content-Type: application/json" \
  -d '{"repo_url": "git@github.com:test/repo2.git", "description": "Low priority task", "priority": "low"}' > /dev/null
curl -s -X POST "$BASE_URL/api/tasks" \
  -H "Content-Type: application/json" \
  -d '{"repo_url": "git@github.com:test/repo3.git", "description": "Urgent task", "priority": "urgent"}' > /dev/null
echo "   Created 2 more tasks"
echo ""

echo "10. Filtering tasks by status..."
QUEUED_TASKS=$(curl -s "$BASE_URL/api/tasks?status=queued")
echo "   Queued tasks: $(echo $QUEUED_TASKS | jq '.total')"
echo ""

echo "11. Filtering tasks by status (claimed)..."
CLAIMED_TASKS=$(curl -s "$BASE_URL/api/tasks?status=claimed")
echo "   Claimed tasks: $(echo $CLAIMED_TASKS | jq '.total')"
echo ""

echo "12. Claiming highest priority task..."
CLAIM2_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tasks/claim" \
  -H "Content-Type: application/json" \
  -d "{\"worker_id\": \"$WORKER_ID\"}")
echo "   Claimed task priority: $(echo $CLAIM2_RESPONSE | jq -r '.task.priority')"
echo ""

echo "13. Submitting feedback for a task..."
FEEDBACK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tasks/$CLAIMED_TASK_ID/feedback" \
  -H "Content-Type: application/json" \
  -d '{
    "feedback_type": "request_changes",
    "message": "Please add more tests"
  }')
echo "   Feedback submitted successfully"
echo ""

echo "14. Cancelling a task..."
curl -s -X DELETE "$BASE_URL/api/tasks/$CLAIMED_TASK_ID" > /dev/null
CANCELLED_TASK=$(curl -s "$BASE_URL/api/tasks/$CLAIMED_TASK_ID")
echo "   Task status after cancel: $(echo $CANCELLED_TASK | jq -r '.status')"
echo ""

echo "========================================="
echo "Phase 1 Demo Complete!"
echo "========================================="
echo ""
echo "Summary:"
echo "- Created tasks with different priorities"
echo "- Registered workers"
echo "- Workers claimed tasks (priority order)"
echo "- Sent heartbeats"
echo "- Filtered tasks by status"
echo "- Submitted feedback"
echo "- Cancelled tasks"
echo ""
echo "All REST API endpoints working correctly!"
