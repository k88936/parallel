import { apiClient } from './client';
import type {
    Task,
    TaskListResponse,
    ListTasksQuery,
    ReviewData,
    SubmitFeedbackRequest,
    RetryTaskResponse,
    RetryTaskRequest,
    CreateTaskRequest,
    CreateTaskResponse,
} from '../types';

const TASKS_PATH = '/api/tasks';

export const tasksApi = {
    create: async (data: CreateTaskRequest): Promise<CreateTaskResponse> => {
        const response = await apiClient.post<CreateTaskResponse>(TASKS_PATH, data);
        return response.data;
    },

    list: async (query?: ListTasksQuery): Promise<TaskListResponse> => {
        const response = await apiClient.get<TaskListResponse>(TASKS_PATH, {
            params: query,
        });
        return response.data;
    },

    get: async (id: string): Promise<Task> => {
        const response = await apiClient.get<Task>(`${TASKS_PATH}/${id}`);
        return response.data;
    },

    cancel: async (id: string): Promise<void> => {
        await apiClient.delete(`${TASKS_PATH}/${id}`);
    },

    retry: async (id: string, clearReviewData?: boolean): Promise<RetryTaskResponse> => {
        const body: RetryTaskRequest = {
            clear_review_data: clearReviewData ?? null,
        };
        const response = await apiClient.post<RetryTaskResponse>(`${TASKS_PATH}/${id}/retry`, body);
        return response.data;
    },

    submitFeedback: async (id: string, feedback: SubmitFeedbackRequest): Promise<void> => {
        await apiClient.post(`${TASKS_PATH}/${id}/feedback`, feedback);
    },

    getReviewData: async (id: string): Promise<ReviewData | null> => {
        const response = await apiClient.get<ReviewData | null>(`${TASKS_PATH}/${id}/review`);
        return response.data;
    },
};
