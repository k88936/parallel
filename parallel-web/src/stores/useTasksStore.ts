import {create} from 'zustand';
import {tasksApi} from '../api';
import type {
    ListTasksQuery,
    ReviewData,
    SubmitFeedbackRequest,
    Task,
} from '../types';

interface TasksStoreState {
    tasks: Task[];
    total: number;
    hasMore: boolean;
    reviewData: Record<string, ReviewData>;
    reviewLoadingIds: Record<string, boolean>;
    loading: boolean;
    error: string | null;
    lastQuery: ListTasksQuery | null;
    fetchTasks: (query: ListTasksQuery) => Promise<void>;
    refreshTasks: () => Promise<void>;
    fetchReviewData: (taskId: string) => Promise<void>;
    cancelTask: (taskId: string) => Promise<void>;
    retryTask: (taskId: string, clearReviewData?: boolean) => Promise<void>;
    submitFeedback: (taskId: string, feedback: SubmitFeedbackRequest) => Promise<void>;
    clearError: () => void;
}

const getErrorMessage = (error: unknown): string => {
    if (error instanceof Error) {
        return error.message;
    }

    return 'Request failed';
};

export const useTasksStore = create<TasksStoreState>((set, get) => ({
    tasks: [],
    total: 0,
    hasMore: false,
    reviewData: {},
    reviewLoadingIds: {},
    loading: false,
    error: null,
    lastQuery: null,
    fetchTasks: async (query) => {
        set({loading: true, error: null, lastQuery: query});
        try {
            const response = await tasksApi.list(query);
            set({
                tasks: response.tasks,
                total: response.total,
                hasMore: response.has_more,
                loading: false,
            });
        } catch (error) {
            set({loading: false, error: getErrorMessage(error)});
            throw error;
        }
    },
    refreshTasks: async () => {
        const {lastQuery, fetchTasks} = get();
        if (!lastQuery) {
            return;
        }

        await fetchTasks(lastQuery);
    },
    fetchReviewData: async (taskId) => {
        if (get().reviewData[taskId] || get().reviewLoadingIds[taskId]) {
            return;
        }

        set((state) => ({
            reviewLoadingIds: {
                ...state.reviewLoadingIds,
                [taskId]: true,
            },
            error: null,
        }));

        try {
            const reviewData = await tasksApi.getReviewData(taskId);
            set((state) => ({
                reviewData: reviewData
                    ? {
                        ...state.reviewData,
                        [taskId]: reviewData,
                    }
                    : state.reviewData,
                reviewLoadingIds: {
                    ...state.reviewLoadingIds,
                    [taskId]: false,
                },
            }));
        } catch (error) {
            set((state) => ({
                error: getErrorMessage(error),
                reviewLoadingIds: {
                    ...state.reviewLoadingIds,
                    [taskId]: false,
                },
            }));
            throw error;
        }
    },
    cancelTask: async (taskId) => {
        set({error: null});
        try {
            await tasksApi.cancel(taskId);
            await get().refreshTasks();
        } catch (error) {
            set({error: getErrorMessage(error)});
            throw error;
        }
    },
    retryTask: async (taskId, clearReviewData = false) => {
        set({error: null});
        try {
            await tasksApi.retry(taskId, clearReviewData);
            set((state) => {
                if (!clearReviewData) {
                    return state;
                }

                const nextReviewData = {...state.reviewData};
                delete nextReviewData[taskId];
                return {reviewData: nextReviewData};
            });
            await get().refreshTasks();
        } catch (error) {
            set({error: getErrorMessage(error)});
            throw error;
        }
    },
    submitFeedback: async (taskId, feedback) => {
        set({error: null});
        try {
            await tasksApi.submitFeedback(taskId, feedback);
            await get().refreshTasks();
        } catch (error) {
            set({error: getErrorMessage(error)});
            throw error;
        }
    },
    clearError: () => set({error: null}),
}));
