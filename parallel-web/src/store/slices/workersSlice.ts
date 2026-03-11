import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type { WorkerSummary, WorkerInfo, ResourceMonitor } from '../../types';
import { workersApi } from '../../api/workers';

interface WorkersState {
    workers: WorkerSummary[];
    selectedWorkerId: string | null;
    selectedWorkerInfo: WorkerInfo | null;
    selectedWorkerResources: ResourceMonitor | null;
    loading: boolean;
    infoLoading: boolean;
    error: string | null;
}

const initialState: WorkersState = {
    workers: [],
    selectedWorkerId: null,
    selectedWorkerInfo: null,
    selectedWorkerResources: null,
    loading: false,
    infoLoading: false,
    error: null,
};

export const fetchWorkers = createAsyncThunk(
    'workers/fetchAll',
    async () => {
        const workers = await workersApi.list();
        return workers;
    }
);

export const fetchWorkerInfo = createAsyncThunk(
    'workers/fetchInfo',
    async (workerId: string) => {
        const info = await workersApi.getInfo(workerId);
        return info;
    }
);

export const fetchWorkerResources = createAsyncThunk(
    'workers/fetchResources',
    async (workerId: string) => {
        const resources = await workersApi.getResources(workerId);
        return resources;
    }
);

const workersSlice = createSlice({
    name: 'workers',
    initialState,
    reducers: {
        selectWorker: (state, action: PayloadAction<string | null>) => {
            state.selectedWorkerId = action.payload;
            if (!action.payload) {
                state.selectedWorkerInfo = null;
                state.selectedWorkerResources = null;
            }
        },
        clearError: (state) => {
            state.error = null;
        },
    },
    extraReducers: (builder) => {
        builder
            .addCase(fetchWorkers.pending, (state) => {
                state.loading = true;
                state.error = null;
            })
            .addCase(fetchWorkers.fulfilled, (state, action) => {
                state.loading = false;
                state.workers = action.payload;
            })
            .addCase(fetchWorkers.rejected, (state, action) => {
                state.loading = false;
                state.error = action.error.message || 'Failed to fetch workers';
            })
            .addCase(fetchWorkerInfo.pending, (state) => {
                state.infoLoading = true;
            })
            .addCase(fetchWorkerInfo.fulfilled, (state, action) => {
                state.infoLoading = false;
                state.selectedWorkerInfo = action.payload;
            })
            .addCase(fetchWorkerInfo.rejected, (state, action) => {
                state.infoLoading = false;
                state.error = action.error.message || 'Failed to fetch worker info';
            })
            .addCase(fetchWorkerResources.fulfilled, (state, action) => {
                state.selectedWorkerResources = action.payload;
            });
    },
});

export const { selectWorker, clearError } = workersSlice.actions;
export default workersSlice.reducer;
