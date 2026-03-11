import { apiClient } from './client';
import type { WorkerSummary, WorkerInfo, ResourceMonitor } from '../types';

const WORKERS_PATH = '/api/workers';

export const workersApi = {
    list: async (): Promise<WorkerSummary[]> => {
        const response = await apiClient.get<WorkerSummary[]>(WORKERS_PATH);
        return response.data;
    },

    getInfo: async (id: string): Promise<WorkerInfo> => {
        const response = await apiClient.get<WorkerInfo>(`${WORKERS_PATH}/${id}/info`);
        return response.data;
    },

    getResources: async (id: string): Promise<ResourceMonitor> => {
        const response = await apiClient.get<ResourceMonitor>(`${WORKERS_PATH}/${id}/resources`);
        return response.data;
    },
};
