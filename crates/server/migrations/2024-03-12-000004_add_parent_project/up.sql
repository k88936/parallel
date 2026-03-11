ALTER TABLE projects ADD COLUMN parent_id TEXT;

INSERT INTO projects (id, name, repos_json, ssh_keys_json, created_at, updated_at, parent_id)
VALUES (
    '00000000-0000-0000-0000-000000000000',
    'Root',
    '[]',
    '[]',
    datetime('now'),
    datetime('now'),
    NULL
);
