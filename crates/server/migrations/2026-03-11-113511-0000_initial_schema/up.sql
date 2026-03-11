-- Your SQL goes here
CREATE TABLE `workers`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`token` TEXT NOT NULL,
	`name` TEXT NOT NULL,
	`status` TEXT NOT NULL,
	`last_heartbeat` TIMESTAMP NOT NULL,
	`current_tasks_json` TEXT NOT NULL,
	`pending_instructions_json` TEXT NOT NULL,
	`capabilities_json` TEXT NOT NULL,
	`max_concurrent` INTEGER NOT NULL
);

CREATE TABLE `tasks`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`title` TEXT NOT NULL,
	`repo_url` TEXT NOT NULL,
	`description` TEXT NOT NULL,
	`base_branch` TEXT NOT NULL,
	`target_branch` TEXT NOT NULL,
	`status` TEXT NOT NULL,
	`priority` INTEGER NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`claimed_by` TEXT,
	`review_data_json` TEXT,
	`ssh_key` TEXT NOT NULL,
	`max_execution_time` BIGINT NOT NULL,
	`project_id` TEXT,
	`required_labels_json` TEXT NOT NULL
);

CREATE TABLE `projects`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`repos_json` TEXT NOT NULL,
	`ssh_keys_json` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`parent_id` TEXT
);

INSERT OR IGNORE INTO projects (id, name, repos_json, ssh_keys_json, created_at, updated_at, parent_id)
VALUES ('root', 'Root', '[]', '[]', datetime('now'), datetime('now'), NULL);
