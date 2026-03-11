import { createSlice, createAsyncThunk, type PayloadAction } from '@reduxjs/toolkit';
import type { Project, CreateProjectRequest, UpdateProjectRequest } from '../../types';
import { projectsApi } from '../../api';

interface ProjectsState {
    projects: Record<string, Project>;
    rootProjectId: string | null;
    childrenByParent: Record<string, string[]>;
    expandedNodes: string[];
    selectedProjectId: string | null;
    loading: boolean;
    error: string | null;
}

const initialState: ProjectsState = {
    projects: {},
    rootProjectId: null,
    childrenByParent: {},
    expandedNodes: [],
    selectedProjectId: null,
    loading: false,
    error: null,
};

export const fetchRootProject = createAsyncThunk(
    'projects/fetchRoot',
    async () => {
        const project = await projectsApi.getRoot();
        return project;
    }
);

export const fetchProjectChildren = createAsyncThunk(
    'projects/fetchChildren',
    async (projectId: string) => {
        const children = await projectsApi.getChildren(projectId);
        return { projectId, children };
    }
);

export const createProject = createAsyncThunk(
    'projects/create',
    async (data: CreateProjectRequest) => {
        const response = await projectsApi.create(data);
        const project = await projectsApi.get(response.project_id);
        return project;
    }
);

export const updateProject = createAsyncThunk(
    'projects/update',
    async ({ id, data }: { id: string; data: UpdateProjectRequest }) => {
        const project = await projectsApi.update(id, data);
        return project;
    }
);

export const deleteProject = createAsyncThunk(
    'projects/delete',
    async (id: string) => {
        await projectsApi.delete(id);
        return id;
    }
);

const projectsSlice = createSlice({
    name: 'projects',
    initialState,
    reducers: {
        toggleNode: (state, action: PayloadAction<string>) => {
            const nodeId = action.payload;
            const index = state.expandedNodes.indexOf(nodeId);
            if (index >= 0) {
                state.expandedNodes.splice(index, 1);
            } else {
                state.expandedNodes.push(nodeId);
            }
        },
        expandNode: (state, action: PayloadAction<string>) => {
            const nodeId = action.payload;
            if (!state.expandedNodes.includes(nodeId)) {
                state.expandedNodes.push(nodeId);
            }
        },
        collapseNode: (state, action: PayloadAction<string>) => {
            const nodeId = action.payload;
            const index = state.expandedNodes.indexOf(nodeId);
            if (index >= 0) {
                state.expandedNodes.splice(index, 1);
            }
        },
        selectProject: (state, action: PayloadAction<string | null>) => {
            state.selectedProjectId = action.payload;
        },
        clearError: (state) => {
            state.error = null;
        },
    },
    extraReducers: (builder) => {
        builder
            .addCase(fetchRootProject.pending, (state) => {
                state.loading = true;
                state.error = null;
            })
            .addCase(fetchRootProject.fulfilled, (state, action) => {
                state.loading = false;
                const project = action.payload;
                state.projects[project.id] = project;
                state.rootProjectId = project.id;
                state.selectedProjectId = project.id;
            })
            .addCase(fetchRootProject.rejected, (state, action) => {
                state.loading = false;
                state.error = action.error.message || 'Failed to fetch root project';
            })
            .addCase(fetchProjectChildren.pending, (state) => {
                state.loading = true;
            })
            .addCase(fetchProjectChildren.fulfilled, (state, action) => {
                state.loading = false;
                const { projectId, children } = action.payload;
                state.childrenByParent[projectId] = children.map(c => c.id);
                children.forEach(child => {
                    state.projects[child.id] = child;
                });
            })
            .addCase(fetchProjectChildren.rejected, (state, action) => {
                state.loading = false;
                state.error = action.error.message || 'Failed to fetch children';
            })
            .addCase(createProject.fulfilled, (state, action) => {
                const project = action.payload;
                state.projects[project.id] = project;
                if (project.parent_id) {
                    if (!state.childrenByParent[project.parent_id]) {
                        state.childrenByParent[project.parent_id] = [];
                    }
                    state.childrenByParent[project.parent_id].push(project.id);
                }
            })
            .addCase(updateProject.fulfilled, (state, action) => {
                const project = action.payload;
                state.projects[project.id] = project;
            })
            .addCase(deleteProject.fulfilled, (state, action) => {
                const id = action.payload;
                const project = state.projects[id];
                if (project?.parent_id) {
                    const siblings = state.childrenByParent[project.parent_id];
                    if (siblings) {
                        const index = siblings.indexOf(id);
                        if (index >= 0) {
                            siblings.splice(index, 1);
                        }
                    }
                }
                delete state.projects[id];
                delete state.childrenByParent[id];
                if (state.selectedProjectId === id) {
                    state.selectedProjectId = state.rootProjectId;
                }
            });
    },
});

export const { toggleNode, expandNode, collapseNode, selectProject, clearError } = projectsSlice.actions;
export default projectsSlice.reducer;
