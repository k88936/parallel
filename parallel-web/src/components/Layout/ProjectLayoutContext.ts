import {useOutletContext} from 'react-router-dom';
import type {Project} from '../../types';

export interface ProjectLayoutContextValue {
    projects: Record<string, Project>;
    childrenByParent: Record<string, string[]>;
    selectedProjectId: string | null;
    expandedNodes: string[];
    loading: boolean;
    error: string | null;
    ensureProjectLoaded: (projectId: string) => Promise<Project>;
    ensureChildrenLoaded: (projectId: string) => Promise<void>;
    refreshProject: (projectId: string) => Promise<Project>;
    refreshChildren: (projectId: string) => Promise<void>;
    upsertProject: (project: Project) => void;
    removeProject: (projectId: string, parentId?: string) => void;
    expandNodes: (projectIds: string[]) => void;
}

export const useProjectLayoutContext = () => useOutletContext<ProjectLayoutContextValue>();
