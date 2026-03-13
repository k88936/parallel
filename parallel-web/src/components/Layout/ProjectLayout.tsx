import {useCallback, useEffect, useMemo, useRef, useState} from 'react';
import {Outlet, useParams} from 'react-router-dom';
import {projectsApi} from '../../api';
import type {Project} from '../../types';
import {ProjectSidebar} from './ProjectSidebar.tsx';
import type {ProjectLayoutContextValue} from './ProjectLayoutContext';
import Group from '@jetbrains/ring-ui-built/components/group/group';

const getErrorMessage = (error: unknown): string => {
    if (error instanceof Error) {
        return error.message;
    }

    return 'Request failed';
};

export const ProjectLayout = () => {
    const {projectId} = useParams<{projectId: string}>();
    const [projects, setProjects] = useState<Record<string, Project>>({});
    const [childrenByParent, setChildrenByParent] = useState<Record<string, string[]>>({});
    const [expandedNodes, setExpandedNodes] = useState<string[]>(['root']);
    const [pendingRequests, setPendingRequests] = useState(0);
    const [error, setError] = useState<string | null>(null);

    const projectsRef = useRef(projects);
    const childrenRef = useRef(childrenByParent);

    useEffect(() => {
        projectsRef.current = projects;
    }, [projects]);

    useEffect(() => {
        childrenRef.current = childrenByParent;
    }, [childrenByParent]);

    const startRequest = useCallback(() => {
        setPendingRequests((current) => current + 1);
    }, []);

    const finishRequest = useCallback(() => {
        setPendingRequests((current) => Math.max(0, current - 1));
    }, []);

    const upsertProject = useCallback((project: Project) => {
        setProjects((current) => ({
            ...current,
            [project.id]: project,
        }));
    }, []);

    const expandNodes = useCallback((projectIds: string[]) => {
        if (projectIds.length === 0) {
            return;
        }

        setExpandedNodes((current) => Array.from(new Set([...current, ...projectIds])));
    }, []);

    const refreshProject = useCallback(async (targetProjectId: string) => {
        startRequest();
        try {
            const project = await projectsApi.get(targetProjectId);
            setError(null);
            upsertProject(project);
            return project;
        } catch (error) {
            const message = getErrorMessage(error);
            setError(message);
            throw new Error(message);
        } finally {
            finishRequest();
        }
    }, [finishRequest, startRequest, upsertProject]);

    const ensureProjectLoaded = useCallback(async (targetProjectId: string) => {
        const existingProject = projectsRef.current[targetProjectId];
        if (existingProject) {
            return existingProject;
        }

        return refreshProject(targetProjectId);
    }, [refreshProject]);

    const refreshChildren = useCallback(async (targetProjectId: string) => {
        startRequest();
        try {
            const children = await projectsApi.getChildren(targetProjectId);
            setError(null);
            setProjects((current) => {
                const nextProjects = {...current};
                children.forEach((child) => {
                    nextProjects[child.id] = child;
                });
                return nextProjects;
            });
            setChildrenByParent((current) => ({
                ...current,
                [targetProjectId]: children.map((child) => child.id),
            }));
        } catch (error) {
            const message = getErrorMessage(error);
            setError(message);
            throw new Error(message);
        } finally {
            finishRequest();
        }
    }, [finishRequest, startRequest]);

    const ensureChildrenLoaded = useCallback(async (targetProjectId: string) => {
        if (Object.prototype.hasOwnProperty.call(childrenRef.current, targetProjectId)) {
            return;
        }

        await refreshChildren(targetProjectId);
    }, [refreshChildren]);

    const removeProject = useCallback((targetProjectId: string, parentId?: string) => {
        setProjects((current) => {
            const nextProjects = {...current};
            delete nextProjects[targetProjectId];
            return nextProjects;
        });

        setChildrenByParent((current) => {
            const nextChildren = {...current};
            delete nextChildren[targetProjectId];
            if (parentId && nextChildren[parentId]) {
                nextChildren[parentId] = nextChildren[parentId].filter((childId) => childId !== targetProjectId);
            }
            return nextChildren;
        });

        setExpandedNodes((current) => current.filter((projectNodeId) => projectNodeId !== targetProjectId));
    }, []);

    useEffect(() => {
        let isCancelled = false;

        const loadRoot = async () => {
            startRequest();
            try {
                const rootProject = await projectsApi.getRoot();
                if (isCancelled) {
                    return;
                }

                setError(null);
                upsertProject(rootProject);
                expandNodes([rootProject.id]);
            } catch (error) {
                if (!isCancelled) {
                    setError(getErrorMessage(error));
                }
            } finally {
                if (!isCancelled) {
                    finishRequest();
                }
            }
        };

        void loadRoot();

        return () => {
            isCancelled = true;
        };
    }, [expandNodes, finishRequest, startRequest, upsertProject]);

    const contextValue = useMemo<ProjectLayoutContextValue>(() => ({
        projects,
        childrenByParent,
        selectedProjectId: projectId ?? null,
        expandedNodes,
        loading: pendingRequests > 0,
        error,
        ensureProjectLoaded,
        ensureChildrenLoaded,
        refreshProject,
        refreshChildren,
        upsertProject,
        removeProject,
        expandNodes,
    }), [
        childrenByParent,
        ensureChildrenLoaded,
        ensureProjectLoaded,
        error,
        expandNodes,
        expandedNodes,
        pendingRequests,
        projectId,
        projects,
        refreshChildren,
        refreshProject,
        removeProject,
        upsertProject,
    ]);

    return (
        <Group className="flex flex-1 overflow-hidden">
            <ProjectSidebar
                projects={projects}
                childrenByParent={childrenByParent}
                selectedProjectId={projectId ?? null}
                expandedNodes={expandedNodes}
                loading={pendingRequests > 0 && !projects.root}
                error={error}
                onNodeToggle={(targetProjectId) => {
                    setExpandedNodes((current) =>
                        current.includes(targetProjectId)
                            ? current.filter((nodeId) => nodeId !== targetProjectId)
                            : [...current, targetProjectId],
                    );
                }}
                onLoadChildren={ensureChildrenLoaded}
            />
            <Group className="flex-1 flex flex-col overflow-hidden p-4">
                <Outlet context={contextValue} />
            </Group>
        </Group>
    );
};
