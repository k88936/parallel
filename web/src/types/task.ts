export type TaskStatus =
    | 'created'
    | 'queued'
    | 'claimed'
    | 'in_progress'
    | 'awaiting_review'
    | 'pending_rework'
    | 'completed'
    | 'cancelled'
    | 'failed';

export type TaskPriority = 'low' | 'normal' | 'high' | 'urgent';

export type IterationStatus = 'success' | 'failed' | 'cancelled';

export type FeedbackType = 'approve' | 'request_changes' | 'abort';

export interface AgentMessage {
    timestamp: string;
    role: string;
    content: string;
}

export interface IterationResult {
    status: IterationStatus;
    summary: string;
    files_changed: string[];
    commits: string[];
    agent_messages: AgentMessage[];
}

export interface HumanFeedback {
    provided_at: string;
    feedback_type: FeedbackType;
    message: string;
}

export interface TaskIteration {
    iteration_id: number;
    started_at: string;
    completed_at: string | null;
    result: IterationResult | null;
    human_feedback: HumanFeedback | null;
}

export interface Task {
    id: string;
    repo_url: string;
    description: string;
    base_branch: string;
    target_branch: string;
    status: TaskStatus;
    priority: TaskPriority;
    created_at: string;
    updated_at: string;
    claimed_by: string | null;
}

export interface CreateTaskRequest {
    repo_url: string;
    description: string;
    base_branch?: string;
    target_branch?: string;
    priority?: TaskPriority;
}

export interface CreateTaskResponse {
    task_id: string;
}

export interface ListTasksQuery {
    status?: TaskStatus;
    limit?: number;
    offset?: number;
}

export interface TaskListResponse {
    tasks: Task[];
    total: number;
}

export interface SubmitFeedbackRequest {
    feedback_type: FeedbackType;
    message: string;
}

export interface ReviewData {
    messages: AgentMessage[];
    diff: string;
}
