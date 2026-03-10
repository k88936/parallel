CREATE TABLE tasks (
    id TEXT NOT NULL PRIMARY KEY,
    title TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    description TEXT NOT NULL,
    base_branch TEXT NOT NULL,
    target_branch TEXT NOT NULL,
    status TEXT NOT NULL,
    priority INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    claimed_by TEXT,
    review_data_json TEXT,
    ssh_key TEXT NOT NULL DEFAULT '',
    max_execution_time BIGINT NOT NULL DEFAULT 3600,
    project_id TEXT,
    required_labels_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_claimed_by ON tasks(claimed_by);
