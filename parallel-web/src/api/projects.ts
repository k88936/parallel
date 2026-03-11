import { apiClient } from './client';
import type {
    Project,
    CreateProjectRequest,
    CreateProjectResponse,
    UpdateProjectRequest,
    ListProjectsQuery,
    ProjectListResponse,
} from '../types';

const PROJECTS_PATH = '/api/projects';

export const projectsApi = {
    getRoot: async (): Promise<Project> => {
        const response = await apiClient.get<Project>(`${PROJECTS_PATH}/root`);
        return response.data;
    },

    list: async (query?: ListProjectsQuery): Promise<ProjectListResponse> => {
        const response = await apiClient.get<ProjectListResponse>(PROJECTS_PATH, {
            params: query,
        });
        return response.data;
    },

    get: async (id: string): Promise<Project> => {
        const response = await apiClient.get<Project>(`${PROJECTS_PATH}/${id}`);
        return response.data;
    },

    getChildren: async (id: string): Promise<Project[]> => {
        const response = await apiClient.get<Project[]>(`${PROJECTS_PATH}/${id}/children`);
        return response.data;
    },

    create: async (data: CreateProjectRequest): Promise<CreateProjectResponse> => {
        const response = await apiClient.post<CreateProjectResponse>(PROJECTS_PATH, data);
        return response.data;
    },

    update: async (id: string, data: UpdateProjectRequest): Promise<Project> => {
        const response = await apiClient.put<Project>(`${PROJECTS_PATH}/${id}`, data);
        return response.data;
    },

    delete: async (id: string): Promise<void> => {
        await apiClient.delete(`${PROJECTS_PATH}/${id}`);
    },
};
