export type AlertSeverity = 'info' | 'warning' | 'error' | 'critical';

export interface WorkerOfflineAlert {
    type: 'worker_offline';
    worker_id: string;
    worker_name: string;
    running_tasks: string[];
    timestamp: string;
}

export interface WorkerOnlineAlert {
    type: 'worker_online';
    worker_id: string;
    worker_name: string;
    timestamp: string;
}

export interface TaskTimeoutAlert {
    type: 'task_timeout';
    task_id: string;
    task_title: string;
    max_execution_time: number;
    timestamp: string;
}

export interface TaskReviewRequestedAlert {
    type: 'task_review_requested';
    task_id: string;
    task_title: string;
    worker_id: string;
    timestamp: string;
}

export interface TaskCompletedAlert {
    type: 'task_completed';
    task_id: string;
    task_title: string;
    timestamp: string;
}

export interface TaskFailedAlert {
    type: 'task_failed';
    task_id: string;
    task_title: string;
    error: string;
    timestamp: string;
}

export interface TaskCancelledAlert {
    type: 'task_cancelled';
    task_id: string;
    task_title: string;
    timestamp: string;
}

export type Alert =
    | WorkerOfflineAlert
    | WorkerOnlineAlert
    | TaskTimeoutAlert
    | TaskReviewRequestedAlert
    | TaskCompletedAlert
    | TaskFailedAlert
    | TaskCancelledAlert;

export interface AlertPayload {
    alert: Alert;
    severity: AlertSeverity;
}

export interface AlertWithId extends AlertPayload {
    id: string;
}
