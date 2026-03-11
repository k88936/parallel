import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type {
    Task,
    TaskListResponse,
    ListTasksQuery,
    TaskStatus,
    TaskPriority,
    ReviewData,
    SubmitFeedbackRequest,
    CreateTaskRequest,
} from '../../types';
import { tasksApi } from '../../api';

interface TasksState {
    tasks: Task[];
    total: number;
    hasMore: boolean;
    expandedTaskIds: string[];
    filters: {
        status?: TaskStatus;
        priority?: TaskPriority;
        search?: string;
        worker_id?: string;
        project_id?: string;
    };
    pagination: {
        limit: number;
        offset: number;
    };
    reviewData: Record<string, ReviewData>;
    loading: boolean;
    reviewLoading: boolean;
    createLoading: boolean;
    createError: string | null;
    error: string | null;
}

const initialState: TasksState = {
    tasks: [],
    total: 0,
    hasMore: false,
    expandedTaskIds: [],
    filters: {},
    pagination: {
        limit: 20,
        offset: 0,
    },
    reviewData: {},
    loading: false,
    reviewLoading: false,
    createLoading: false,
    createError: null,
    error: null,
};

export const fetchTasks = createAsyncThunk(
    'tasks/fetchList',
    async (query?: Partial<ListTasksQuery>) => {
        const response = await tasksApi.list(query as ListTasksQuery);
        return response;
    }
);

export const fetchTask = createAsyncThunk(
    'tasks/fetchOne',
    async (id: string) => {
        const task = await tasksApi.get(id);
        return task;
    }
);

export const cancelTask = createAsyncThunk(
    'tasks/cancel',
    async (id: string) => {
        await tasksApi.cancel(id);
        return id;
    }
);

export const retryTask = createAsyncThunk(
    'tasks/retry',
    async ({ id, clearReviewData }: { id: string; clearReviewData?: boolean }) => {
        const response = await tasksApi.retry(id, clearReviewData);
        return response;
    }
);

export const fetchReviewData = createAsyncThunk(
    'tasks/fetchReviewData',
    async (id: string) => {
        const data = await tasksApi.getReviewData(id);
        return { id, data };
    }
);

export const submitFeedback = createAsyncThunk(
    'tasks/submitFeedback',
    async ({ id, feedback }: { id: string; feedback: SubmitFeedbackRequest }) => {
        await tasksApi.submitFeedback(id, feedback);
        return { id, feedbackType: feedback.feedback_type };
    }
);

export const createTask = createAsyncThunk(
    'tasks/create',
    async (data: CreateTaskRequest) => {
        const response = await tasksApi.create(data);
        return response;
    }
);

const tasksSlice = createSlice({
    name: 'tasks',
    initialState,
    reducers: {
        toggleExpand: (state, action: PayloadAction<string>) => {
            const taskId = action.payload;
            const index = state.expandedTaskIds.indexOf(taskId);
            if (index >= 0) {
                state.expandedTaskIds.splice(index, 1);
            } else {
                state.expandedTaskIds.push(taskId);
            }
        },
        expandTask: (state, action: PayloadAction<string>) => {
            const taskId = action.payload;
            if (!state.expandedTaskIds.includes(taskId)) {
                state.expandedTaskIds.push(taskId);
            }
        },
        collapseTask: (state, action: PayloadAction<string>) => {
            const taskId = action.payload;
            const index = state.expandedTaskIds.indexOf(taskId);
            if (index >= 0) {
                state.expandedTaskIds.splice(index, 1);
            }
        },
        collapseAll: (state) => {
            state.expandedTaskIds = [];
        },
        setFilters: (state, action: PayloadAction<Partial<TasksState['filters']>>) => {
            state.filters = { ...state.filters, ...action.payload };
            state.pagination.offset = 0;
        },
        clearFilters: (state) => {
            state.filters = {};
            state.pagination.offset = 0;
        },
        setPagination: (state, action: PayloadAction<Partial<TasksState['pagination']>>) => {
            state.pagination = { ...state.pagination, ...action.payload };
        },
        setPage: (state, action: PayloadAction<number>) => {
            state.pagination.offset = action.payload * state.pagination.limit;
        },
        clearError: (state) => {
            state.error = null;
        },
        clearCreateError: (state) => {
            state.createError = null;
        },
    },
    extraReducers: (builder) => {
        builder
            .addCase(fetchTasks.pending, (state) => {
                state.loading = true;
                state.error = null;
            })
            .addCase(fetchTasks.fulfilled, (state, action) => {
                state.loading = false;
                const response = action.payload as TaskListResponse;
                state.tasks = response.tasks;
                state.total = Number(response.total);
                state.hasMore = response.has_more;
            })
            .addCase(fetchTasks.rejected, (state, action) => {
                state.loading = false;
                state.error = action.error.message || 'Failed to fetch tasks';
            })
            .addCase(fetchTask.fulfilled, (state, action) => {
                const task = action.payload;
                const index = state.tasks.findIndex(t => t.id === task.id);
                if (index >= 0) {
                    state.tasks[index] = task;
                } else {
                    state.tasks.unshift(task);
                }
            })
            .addCase(cancelTask.pending, (state, action) => {
                const taskId = action.meta.arg;
                const task = state.tasks.find(t => t.id === taskId);
                if (task) {
                    task.status = 'cancelled';
                }
            })
            .addCase(cancelTask.rejected, (state, action) => {
                state.error = action.error.message || 'Failed to cancel task';
            })
            .addCase(retryTask.fulfilled, (state, action) => {
                const { task_id, status } = action.payload;
                const task = state.tasks.find(t => t.id === task_id);
                if (task) {
                    task.status = status;
                }
            })
            .addCase(retryTask.rejected, (state, action) => {
                state.error = action.error.message || 'Failed to retry task';
            })
            .addCase(fetchReviewData.pending, (state) => {
                state.reviewLoading = true;
            })
            .addCase(fetchReviewData.fulfilled, (state, action) => {
                state.reviewLoading = false;
                const { id, data } = action.payload;
                if (data) {
                    state.reviewData[id] = data;
                }
            })
            .addCase(fetchReviewData.rejected, (state, action) => {
                state.reviewLoading = false;
                state.error = action.error.message || 'Failed to fetch review data';
            })
            .addCase(submitFeedback.fulfilled, (state, action) => {
                const { id, feedbackType } = action.payload;
                const task = state.tasks.find(t => t.id === id);
                if (task) {
                    if (feedbackType === 'approve') {
                        task.status = 'pending_response';
                    } else if (feedbackType === 'request_changes') {
                        task.status = 'pending_response';
                    } else if (feedbackType === 'abort') {
                        task.status = 'cancelled';
                    }
                }
                const expandIndex = state.expandedTaskIds.indexOf(id);
                if (expandIndex >= 0) {
                    state.expandedTaskIds.splice(expandIndex, 1);
                }
            })
            .addCase(submitFeedback.rejected, (state, action) => {
                state.error = action.error.message || 'Failed to submit feedback';
            })
            .addCase(createTask.pending, (state) => {
                state.createLoading = true;
                state.createError = null;
            })
            .addCase(createTask.fulfilled, (state) => {
                state.createLoading = false;
                state.createError = null;
            })
            .addCase(createTask.rejected, (state, action) => {
                state.createLoading = false;
                state.createError = action.error.message || 'Failed to create task';
            });
    },
});

export const {
    toggleExpand,
    expandTask,
    collapseTask,
    collapseAll,
    setFilters,
    clearFilters,
    setPagination,
    setPage,
    clearError,
    clearCreateError,
} = tasksSlice.actions;

export default tasksSlice.reducer;
