export type TaskStatus =
  | 'created'
  | 'queued'
  | 'claimed'
  | 'in_progress'
  | 'awaiting_review'
  | 'iterating'
  | 'completed'
  | 'cancelled';

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
  iterations: TaskIteration[];
  current_iteration: number;
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

export type ExecutionStage = 'cloning' | 'working' | 'committing' | 'pushing';

export interface TaskProgressUpdate {
  stage: ExecutionStage;
  message: string;
  percentage?: number;
}

export type HumanNotification =
  | { type: 'task_progress'; task_id: string; update: TaskProgressUpdate }
  | { type: 'agent_output'; task_id: string; output: string }
  | { type: 'terminal_output'; task_id: string; terminal_id: string; output: string }
  | { type: 'task_awaiting_review'; task_id: string; result: IterationResult }
  | { type: 'task_completed'; task_id: string; branch: string }
  | { type: 'task_status_update'; task_id: string; status: TaskStatus };

export type HumanMessage =
  | { type: 'send_message'; task_id: string; message: string }
  | { type: 'terminal_input'; task_id: string; terminal_id: string; input: string }
  | { type: 'abort_task'; task_id: string }
  | { type: 'accept_work'; task_id: string };
