CREATE TABLE workers (
    id TEXT NOT NULL PRIMARY KEY,
    token TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    last_heartbeat TIMESTAMP NOT NULL,
    current_tasks_json TEXT NOT NULL DEFAULT '[]',
    pending_instructions_json TEXT NOT NULL DEFAULT '[]',
    capabilities_json TEXT NOT NULL,
    max_concurrent INTEGER NOT NULL
);

CREATE INDEX idx_workers_status ON workers(status);
CREATE INDEX idx_workers_last_heartbeat ON workers(last_heartbeat);
