diesel::table! {
    tasks (id) {
        id -> Text,
        title -> Text,
        repo_url -> Text,
        description -> Text,
        base_branch -> Text,
        target_branch -> Text,
        status -> Text,
        priority -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        claimed_by -> Nullable<Text>,
        review_data_json -> Nullable<Text>,
        ssh_key -> Text,
        max_execution_time -> BigInt,
        project_id -> Nullable<Text>,
        required_labels_json -> Text,
    }
}

diesel::table! {
    workers (id) {
        id -> Text,
        token -> Text,
        name -> Text,
        status -> Text,
        last_heartbeat -> Timestamp,
        current_tasks_json -> Text,
        pending_instructions_json -> Text,
        capabilities_json -> Text,
        max_concurrent -> Integer,
    }
}

diesel::table! {
    projects (id) {
        id -> Text,
        name -> Text,
        repos_json -> Text,
        ssh_keys_json -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(tasks, workers, projects,);
