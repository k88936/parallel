import type {
  Task,
  CreateTaskRequest,
  CreateTaskResponse,
  TaskListResponse,
  ListTasksQuery,
  SubmitFeedbackRequest,
  ReviewData,
} from '@/types/task';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options?: RequestInit
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  async createTask(data: CreateTaskRequest): Promise<CreateTaskResponse> {
    return this.request<CreateTaskResponse>('/api/tasks', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async listTasks(query?: ListTasksQuery): Promise<TaskListResponse> {
    const params = new URLSearchParams();
    if (query?.status) params.append('status', query.status);
    if (query?.limit) params.append('limit', query.limit.toString());
    if (query?.offset) params.append('offset', query.offset.toString());

    const queryString = params.toString();
    const endpoint = queryString ? `/api/tasks?${queryString}` : '/api/tasks';

    return this.request<TaskListResponse>(endpoint);
  }

  async getTask(taskId: string): Promise<Task> {
    return this.request<Task>(`/api/tasks/${taskId}`);
  }

  async cancelTask(taskId: string): Promise<void> {
    return this.request<void>(`/api/tasks/${taskId}`, {
      method: 'DELETE',
    });
  }

  async submitFeedback(
    taskId: string,
    data: SubmitFeedbackRequest
  ): Promise<void> {
    return this.request<void>(`/api/tasks/${taskId}/feedback`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getReviewData(taskId: string): Promise<ReviewData | null> {
    return this.request<ReviewData | null>(`/api/tasks/${taskId}/review`);
  }
}

export const api = new ApiClient(API_BASE);
