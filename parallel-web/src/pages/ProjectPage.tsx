import {useEffect, useMemo, useState} from 'react';
import {useNavigate, useParams} from 'react-router-dom';
import {projectsApi, tasksApi} from '../api';
import {useProjectLayoutContext} from '../components/Layout';
import {TaskQueueView} from '../components/TaskQueueView';
import type {CreateProjectRequest, CreateTaskRequest, Project, RepoConfig, SshKeyConfig} from '../types';

import Breadcrumbs from '@jetbrains/ring-ui-built/components/breadcrumbs/breadcrumbs';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Tabs from '@jetbrains/ring-ui-built/components/tabs/dumb-tabs';
import Tab from '@jetbrains/ring-ui-built/components/tabs/tab';
import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import List from '@jetbrains/ring-ui-built/components/list/list';
import {Type} from '@jetbrains/ring-ui-built/components/list/consts';
import Tag from '@jetbrains/ring-ui-built/components/tag/tag';
import Confirm from '@jetbrains/ring-ui-built/components/confirm/confirm';
import {SubprojectDialog} from '../components/common/SubprojectDialog';
import {SshKeyDialog} from '../components/common/SshKeyDialog';
import {RepoDialog} from '../components/common/RepoDialog';
import {CreateTaskDialog} from '../components/common/CreateTaskDialog';
import Group from "@jetbrains/ring-ui-built/components/group/group";

type TabId = 'overview' | 'settings' | 'repos' | 'tasks';

const getErrorMessage = (error: unknown): string => {
    if (error instanceof Error) {
        return error.message;
    }

    return 'Request failed';
};

const collectAncestorReposAndKeys = (
    projectId: string,
    projects: Record<string, Project>
): { repos: RepoConfig[]; sshKeys: SshKeyConfig[] } => {
    const repos: RepoConfig[] = [];
    const sshKeys: SshKeyConfig[] = [];
    const seenRepos = new Set<string>();
    const seenKeys = new Set<string>();

    let currentId: string | undefined = projectId;

    while (currentId) {
        const currentProject: Project | undefined = projects[currentId];
        if (!currentProject) break;

        currentProject.repos.forEach((repo: RepoConfig) => {
            if (!seenRepos.has(repo.name)) {
                repos.push(repo);
                seenRepos.add(repo.name);
            }
        });

        currentProject.ssh_keys.forEach((key: SshKeyConfig) => {
            if (!seenKeys.has(key.name)) {
                sshKeys.push(key);
                seenKeys.add(key.name);
            }
        });

        currentId = currentProject.parent_id;
    }

    return { repos, sshKeys };
};

export const ProjectPage = () => {
    const {projectId} = useParams<{ projectId: string }>();
    const navigate = useNavigate();
    const {
        projects,
        childrenByParent,
        loading,
        error,
        ensureProjectLoaded,
        ensureChildrenLoaded,
        refreshProject,
        refreshChildren,
        upsertProject,
        removeProject,
        expandNodes,
    } = useProjectLayoutContext();

    const [activeTab, setActiveTab] = useState<TabId>('overview');
    const [showAddDialog, setShowAddDialog] = useState(false);
    const [deleteTarget, setDeleteTarget] = useState<Project | null>(null);
    const [showSshKeyDialog, setShowSshKeyDialog] = useState(false);
    const [editingSshKey, setEditingSshKey] = useState<SshKeyConfig | null>(null);
    const [deleteSshKeyTarget, setDeleteSshKeyTarget] = useState<SshKeyConfig | null>(null);
    const [showRepoDialog, setShowRepoDialog] = useState(false);
    const [editingRepo, setEditingRepo] = useState<RepoConfig | null>(null);
    const [deleteRepoTarget, setDeleteRepoTarget] = useState<RepoConfig | null>(null);
    const [showCreateTaskDialog, setShowCreateTaskDialog] = useState(false);
    const [pageLoading, setPageLoading] = useState(false);
    const [pageError, setPageError] = useState<string | null>(null);
    const [createLoading, setCreateLoading] = useState(false);
    const [createError, setCreateError] = useState<string | null>(null);

    const actualProjectId = projectId;
    const project = actualProjectId ? projects[actualProjectId] : null;
    const children = actualProjectId ? childrenByParent[actualProjectId] || [] : [];

    useEffect(() => {
        let isCancelled = false;

        const loadProjectPage = async () => {
            if (!actualProjectId) {
                return;
            }

            setPageLoading(true);
            try {
                let currentProject = await ensureProjectLoaded(actualProjectId);
                const ancestorIds: string[] = [];

                while (currentProject.parent_id) {
                    ancestorIds.push(currentProject.parent_id);
                    await ensureChildrenLoaded(currentProject.parent_id);
                    currentProject = await ensureProjectLoaded(currentProject.parent_id);
                }

                expandNodes(ancestorIds);
                await ensureChildrenLoaded(actualProjectId);

                if (!isCancelled) {
                    setPageError(null);
                }
            } catch (error) {
                if (!isCancelled) {
                    setPageError(getErrorMessage(error));
                }
            } finally {
                if (!isCancelled) {
                    setPageLoading(false);
                }
            }
        };

        void loadProjectPage();

        return () => {
            isCancelled = true;
        };
    }, [actualProjectId, ensureProjectLoaded, ensureChildrenLoaded, expandNodes]);

    const breadcrumb = useMemo(() => {
        if (!project) {
            return [] as Array<{ name: string; id: string | null }>;
        }

        const parts: Array<{ name: string; id: string | null }> = [];
        let current: Project | null = project;
        while (current) {
            parts.unshift({name: current.name, id: current.id});
            current = current.parent_id ? projects[current.parent_id] ?? null : null;
        }
        return parts;
    }, [project, projects]);

    const runProjectMutation = async <T, >(action: () => Promise<T>): Promise<T> => {
        try {
            setPageError(null);
            return await action();
        } catch (error) {
            const message = getErrorMessage(error);
            setPageError(message);
            throw new Error(message);
        }
    };

    if (!actualProjectId || pageLoading || (loading && !project)) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center gap-4">
                <Loader message="Loading project..."/>
            </div>
        );
    }

    if (!project) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center gap-4">
                <Text>{pageError || error || 'Project not found'}</Text>
            </div>
        );
    }

    const handleCreateSubproject = async (data: CreateProjectRequest) => {
        await runProjectMutation(async () => {
            const response = await projectsApi.create(data);
            await ensureProjectLoaded(response.project_id);
            await refreshChildren(actualProjectId);
        });
    };

    const handleDeleteSubproject = async () => {
        if (!deleteTarget) {
            return;
        }

        await runProjectMutation(async () => {
            await projectsApi.delete(deleteTarget.id);
            removeProject(deleteTarget.id, actualProjectId);
            await refreshChildren(actualProjectId);
            setDeleteTarget(null);
        });
    };

    const handleAddSshKey = async (data: SshKeyConfig) => {
        await runProjectMutation(async () => {
            const updatedProject = await projectsApi.update(actualProjectId, {
                name: null,
                repos: null,
                ssh_keys: [...project.ssh_keys, data],
            });
            upsertProject(updatedProject);
        });
    };

    const handleEditSshKey = async (data: SshKeyConfig) => {
        await runProjectMutation(async () => {
            const updatedProject = await projectsApi.update(actualProjectId, {
                name: null,
                repos: null,
                ssh_keys: project.ssh_keys.map((key) => (key.name === editingSshKey?.name ? data : key)),
            });
            upsertProject(updatedProject);
            setEditingSshKey(null);
        });
    };

    const handleDeleteSshKey = async () => {
        if (!deleteSshKeyTarget) {
            return;
        }

        await runProjectMutation(async () => {
            const updatedProject = await projectsApi.update(actualProjectId, {
                name: null,
                repos: null,
                ssh_keys: project.ssh_keys.filter((key) => key.name !== deleteSshKeyTarget.name),
            });
            upsertProject(updatedProject);
            setDeleteSshKeyTarget(null);
        });
    };

    const handleAddRepo = async (data: RepoConfig) => {
        await runProjectMutation(async () => {
            const updatedProject = await projectsApi.update(actualProjectId, {
                name: null,
                repos: [...project.repos, data],
                ssh_keys: null,
            });
            upsertProject(updatedProject);
        });
    };

    const handleEditRepo = async (data: RepoConfig) => {
        await runProjectMutation(async () => {
            const updatedProject = await projectsApi.update(actualProjectId, {
                name: null,
                repos: project.repos.map((repo) => (repo.name === editingRepo?.name ? data : repo)),
                ssh_keys: null,
            });
            upsertProject(updatedProject);
            setEditingRepo(null);
        });
    };

    const handleDeleteRepo = async () => {
        if (!deleteRepoTarget) {
            return;
        }

        await runProjectMutation(async () => {
            const updatedProject = await projectsApi.update(actualProjectId, {
                name: null,
                repos: project.repos.filter((repo) => repo.name !== deleteRepoTarget.name),
                ssh_keys: null,
            });
            upsertProject(updatedProject);
            setDeleteRepoTarget(null);
        });
    };

    const handleCreateTask = async (data: CreateTaskRequest) => {
        setCreateLoading(true);
        setCreateError(null);
        try {
            await tasksApi.create(data);
            setShowCreateTaskDialog(false);
            await refreshProject(actualProjectId);
        } catch (error) {
            const message = getErrorMessage(error);
            setCreateError(message);
            throw new Error(message);
        } finally {
            setCreateLoading(false);
        }
    };

    return (
        <Group className="flex-1 flex flex-col overflow-hidden p-0">
            {pageError && (
                <Text>{pageError}</Text>
            )}
            <Group className="px-5 py-3 ">
                <Breadcrumbs>
                    {breadcrumb.map((part, index) => (
                        <Button
                            key={part.id || index}
                            inline
                            onClick={() => part.id && index < breadcrumb.length - 1 && navigate(`/projects/${part.id}`)}
                        >
                            {part.name}
                        </Button>
                    ))}
                </Breadcrumbs>
            </Group>

            <Group className="flex justify-between items-center p-5 gap-4">
                <Heading level={1}>{project.name}</Heading>
                <Button primary onClick={() => setShowCreateTaskDialog(true)}>
                    Draft New Task
                </Button>
            </Group>

            <Tabs onSelect={(key) => setActiveTab(key as TabId)} selected={activeTab} className="flex-1 px-5">
                <Tab id="overview" title="Overview">
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>Project Details</Heading>
                        </IslandHeader>
                        <IslandContent>
                            <Group className="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-4">
                                <Group className="flex flex-col gap-2">
                                    <Tag>Parent</Tag>
                                    <Text>{(project.parent_id && projects[project.parent_id]?.name) || 'None'}</Text>
                                </Group>
                                <Group className="flex flex-col gap-2">
                                    <Tag>Subprojects</Tag>
                                    <Text>{children.length}</Text>
                                </Group>
                            </Group>
                        </IslandContent>
                    </Island>

                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>Subprojects</Heading>
                            <Button onClick={() => setShowAddDialog(true)}>Add Subproject</Button>
                        </IslandHeader>
                        <IslandContent>
                            {children.length === 0 ? (
                                <Text>No subprojects yet</Text>
                            ) : (
                                <List
                                    data={children
                                        .map((childId) => {
                                            const child = projects[childId];
                                            if (!child) {
                                                return undefined;
                                            }

                                            return {
                                                rgItemType: Type.ITEM,
                                                key: child.id,
                                                label: child.name,
                                                description: `${child.repos.length} repos`,
                                                onClick: () => navigate(`/projects/${child.id}`),
                                                rightNodes: (
                                                    <Button
                                                        danger
                                                        onClick={(event) => {
                                                            event.stopPropagation();
                                                            setDeleteTarget(child);
                                                        }}
                                                    >
                                                        Delete
                                                    </Button>
                                                ),
                                            };
                                        })
                                        .filter((item): item is NonNullable<typeof item> => item !== undefined)}
                                    onSelect={() => {
                                    }}
                                    onMouseOut={() => {
                                    }}
                                    onScrollToBottom={() => {
                                    }}
                                    onResize={() => {
                                    }}
                                    restoreActiveIndex={false}
                                    activateSingleItem={false}
                                    activateFirstItem={false}
                                    shortcuts={false}
                                    renderOptimization={false}
                                    disableMoveDownOverflow={false}
                                    ariaLabel="Subprojects"
                                />
                            )}
                        </IslandContent>
                    </Island>
                </Tab>
                <Tab id="settings" title="Settings">
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>SSH Keys</Heading>
                            <Button onClick={() => setShowSshKeyDialog(true)}>Add SSH Key</Button>
                        </IslandHeader>
                        <IslandContent>
                            {project.ssh_keys.length === 0 ? (
                                <Text>No SSH keys configured</Text>
                            ) : (
                                <List
                                    data={project.ssh_keys.map((key) => ({
                                        rgItemType: Type.ITEM,
                                        key: key.name,
                                        label: key.name,
                                        description: `${key.key.substring(0, 40)}...`,
                                        rightNodes: (
                                            <>
                                                <Button
                                                    onClick={(event) => {
                                                        event.stopPropagation();
                                                        setEditingSshKey(key);
                                                    }}
                                                >
                                                    Edit
                                                </Button>
                                                <Button
                                                    danger
                                                    onClick={(event) => {
                                                        event.stopPropagation();
                                                        setDeleteSshKeyTarget(key);
                                                    }}
                                                >
                                                    Delete
                                                </Button>
                                            </>
                                        ),
                                    }))}
                                    onSelect={() => {
                                    }}
                                    onMouseOut={() => {
                                    }}
                                    onScrollToBottom={() => {
                                    }}
                                    onResize={() => {
                                    }}
                                    restoreActiveIndex={false}
                                    activateSingleItem={false}
                                    activateFirstItem={false}
                                    shortcuts={false}
                                    renderOptimization={false}
                                    disableMoveDownOverflow={false}
                                    ariaLabel="SSH Keys"
                                />
                            )}
                        </IslandContent>
                    </Island>
                </Tab>
                <Tab id="repos" title="Repositories">
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>Repositories</Heading>
                            <Button onClick={() => setShowRepoDialog(true)}>Add Repository</Button>
                        </IslandHeader>
                        <IslandContent>
                            {project.repos.length === 0 ? (
                                <Text>No repositories configured</Text>
                            ) : (
                                <List
                                    data={project.repos.map((repo) => ({
                                        rgItemType: Type.ITEM,
                                        key: repo.name,
                                        label: repo.name,
                                        description: repo.url,
                                        rightNodes: (
                                            <>
                                                <Button
                                                    onClick={(event) => {
                                                        event.stopPropagation();
                                                        setEditingRepo(repo);
                                                    }}
                                                >
                                                    Edit
                                                </Button>
                                                <Button
                                                    danger
                                                    onClick={(event) => {
                                                        event.stopPropagation();
                                                        setDeleteRepoTarget(repo);
                                                    }}
                                                >
                                                    Delete
                                                </Button>
                                            </>
                                        ),
                                    }))}
                                    onSelect={() => {
                                    }}
                                    onMouseOut={() => {
                                    }}
                                    onScrollToBottom={() => {
                                    }}
                                    onResize={() => {
                                    }}
                                    restoreActiveIndex={false}
                                    activateSingleItem={false}
                                    activateFirstItem={false}
                                    shortcuts={false}
                                    renderOptimization={false}
                                    disableMoveDownOverflow={false}
                                    ariaLabel="Repositories"
                                />
                            )}
                        </IslandContent>
                    </Island>
                </Tab>
                <Tab id="tasks" title="Tasks">
                    <TaskQueueView showHeader={false} projectId={actualProjectId || undefined} />
                </Tab>
            </Tabs>

            <SubprojectDialog
                show={showAddDialog}
                parentId={actualProjectId}
                onClose={() => setShowAddDialog(false)}
                onSubmit={handleCreateSubproject}
            />

            <Confirm
                show={!!deleteTarget}
                text="Delete Subproject"
                description={`Are you sure you want to delete "${deleteTarget?.name}"? This action cannot be undone.`}
                confirmLabel="Delete"
                rejectLabel="Cancel"
                onConfirm={() => void handleDeleteSubproject()}
                onReject={() => setDeleteTarget(null)}
            />

            <SshKeyDialog
                show={showSshKeyDialog}
                onClose={() => setShowSshKeyDialog(false)}
                onSubmit={handleAddSshKey}
            />

            <SshKeyDialog
                show={!!editingSshKey}
                onClose={() => setEditingSshKey(null)}
                onSubmit={handleEditSshKey}
                initialData={editingSshKey}
            />

            <Confirm
                show={!!deleteSshKeyTarget}
                text="Delete SSH Key"
                description={`Are you sure you want to delete SSH key "${deleteSshKeyTarget?.name}"? This action cannot be undone.`}
                confirmLabel="Delete"
                rejectLabel="Cancel"
                onConfirm={() => void handleDeleteSshKey()}
                onReject={() => setDeleteSshKeyTarget(null)}
            />

            <RepoDialog
                show={showRepoDialog}
                onClose={() => setShowRepoDialog(false)}
                onSubmit={handleAddRepo}
            />

            <RepoDialog
                show={!!editingRepo}
                onClose={() => setEditingRepo(null)}
                onSubmit={handleEditRepo}
                initialData={editingRepo}
            />

            <Confirm
                show={!!deleteRepoTarget}
                text="Delete Repository"
                description={`Are you sure you want to delete repository "${deleteRepoTarget?.name}"? This action cannot be undone.`}
                confirmLabel="Delete"
                rejectLabel="Cancel"
                onConfirm={() => void handleDeleteRepo()}
                onReject={() => setDeleteRepoTarget(null)}
            />

            <CreateTaskDialog
                show={showCreateTaskDialog}
                projectId={actualProjectId}
                repos={actualProjectId ? collectAncestorReposAndKeys(actualProjectId, projects).repos : []}
                sshKeys={actualProjectId ? collectAncestorReposAndKeys(actualProjectId, projects).sshKeys : []}
                onClose={() => {
                    setShowCreateTaskDialog(false);
                    setCreateError(null);
                }}
                onSubmit={handleCreateTask}
                loading={createLoading}
                error={createError}
            />
        </Group>
    );
};
