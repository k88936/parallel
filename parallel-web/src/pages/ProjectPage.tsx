import {useEffect, useState} from 'react';
import {useNavigate, useParams} from 'react-router-dom';
import {useAppDispatch, useAppSelector} from '../store/hooks';
import {selectProject, fetchProjectChildren, fetchRootProject, createProject, deleteProject, updateProject} from '../store/slices/projectsSlice';
import type {Project, CreateProjectRequest, SshKeyConfig, RepoConfig} from '../types';
import styles from './ProjectPage.module.css';

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

type TabId = 'overview' | 'settings' | 'repos' | 'tasks';

export const ProjectPage = () => {
    const {projectId} = useParams<{ projectId: string }>();
    const navigate = useNavigate();
    const dispatch = useAppDispatch();
    const {projects, childrenByParent, rootProjectId, loading} = useAppSelector((state) => state.projects);
    const [activeTab, setActiveTab] = useState<TabId>('overview');
    const [showAddDialog, setShowAddDialog] = useState(false);
    const [deleteTarget, setDeleteTarget] = useState<Project | null>(null);
    const [showSshKeyDialog, setShowSshKeyDialog] = useState(false);
    const [editingSshKey, setEditingSshKey] = useState<SshKeyConfig | null>(null);
    const [deleteSshKeyTarget, setDeleteSshKeyTarget] = useState<SshKeyConfig | null>(null);
    const [showRepoDialog, setShowRepoDialog] = useState(false);
    const [editingRepo, setEditingRepo] = useState<RepoConfig | null>(null);
    const [deleteRepoTarget, setDeleteRepoTarget] = useState<RepoConfig | null>(null);

    const actualProjectId = projectId === 'root' ? rootProjectId : projectId;
    const project = actualProjectId ? projects[actualProjectId] : null;
    const children = actualProjectId ? (childrenByParent[actualProjectId] || []) : [];

    useEffect(() => {
        if (projectId === 'root' && !rootProjectId) {
            dispatch(fetchRootProject());
        }
    }, [projectId, rootProjectId, dispatch]);

    useEffect(() => {
        if (actualProjectId) {
            dispatch(selectProject(actualProjectId));
            if (!childrenByParent[actualProjectId]) {
                dispatch(fetchProjectChildren(actualProjectId));
            }
        }
    }, [actualProjectId, dispatch, childrenByParent]);

    if (!project) {
        return (
            <div className={styles.loading}>
                <Loader message=
                                  {loading ? 'Loading project...' : 'Project not found'}/>
            </div>
        )
    }

    const getBreadcrumb = () => {
        const parts: { name: string; id: string | null }[] = [];
        let current: Project | null = project;
        while (current) {
            parts.unshift({name: current.name, id: current.id});
            current = current.parent_id ? projects[current.parent_id] : null;
        }
        return parts;
    };

    const breadcrumb = getBreadcrumb();

    const handleCreateSubproject = async (data: CreateProjectRequest) => {
        await dispatch(createProject(data)).unwrap();
    };

    const handleDeleteSubproject = async () => {
        if (deleteTarget) {
            await dispatch(deleteProject(deleteTarget.id)).unwrap();
            setDeleteTarget(null);
        }
    };

    const handleAddSshKey = async (data: SshKeyConfig) => {
        const updatedKeys = [...project.ssh_keys, data];
        await dispatch(updateProject({
            id: actualProjectId!,
            data: {name: null, repos: null, ssh_keys: updatedKeys}
        })).unwrap();
    };

    const handleEditSshKey = async (data: SshKeyConfig) => {
        const updatedKeys = project.ssh_keys.map(k => 
            k.name === editingSshKey?.name ? data : k
        );
        await dispatch(updateProject({
            id: actualProjectId!,
            data: {name: null, repos: null, ssh_keys: updatedKeys}
        })).unwrap();
        setEditingSshKey(null);
    };

    const handleDeleteSshKey = async () => {
        if (deleteSshKeyTarget) {
            const updatedKeys = project.ssh_keys.filter(k => k.name !== deleteSshKeyTarget.name);
            await dispatch(updateProject({
                id: actualProjectId!,
                data: {name: null, repos: null, ssh_keys: updatedKeys}
            })).unwrap();
            setDeleteSshKeyTarget(null);
        }
    };

    const handleAddRepo = async (data: RepoConfig) => {
        const updatedRepos = [...project.repos, data];
        await dispatch(updateProject({
            id: actualProjectId!,
            data: {name: null, repos: updatedRepos, ssh_keys: null}
        })).unwrap();
    };

    const handleEditRepo = async (data: RepoConfig) => {
        const updatedRepos = project.repos.map(r => 
            r.name === editingRepo?.name ? data : r
        );
        await dispatch(updateProject({
            id: actualProjectId!,
            data: {name: null, repos: updatedRepos, ssh_keys: null}
        })).unwrap();
        setEditingRepo(null);
    };

    const handleDeleteRepo = async () => {
        if (deleteRepoTarget) {
            const updatedRepos = project.repos.filter(r => r.name !== deleteRepoTarget.name);
            await dispatch(updateProject({
                id: actualProjectId!,
                data: {name: null, repos: updatedRepos, ssh_keys: null}
            })).unwrap();
            setDeleteRepoTarget(null);
        }
    };

    return (
        <div className={styles.container}>
            <div className={styles.breadcrumbWrapper}>
                <Breadcrumbs>
                    {breadcrumb.map((part, i) => (
                        <span key={part.id || i}>
                            <span
                                className={styles.breadcrumbLink}
                                onClick={() => part.id && i < breadcrumb.length - 1 && navigate(`/projects/${part.id}`)}
                            >
                                {part.name}
                            </span>
                        </span>
                    ))}
                </Breadcrumbs>
            </div>

            <div className={styles.header}>
                <Heading level={1}>{project.name}</Heading>
                <Button primary onClick={() => {}}>
                    Draft New Task
                </Button>
            </div>

            <Tabs
                onSelect={(key) => setActiveTab(key as TabId)}
                selected={activeTab}
                className={styles.tabs}
            >
                <Tab id="overview" title="Overview">
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>Project Details</Heading>
                        </IslandHeader>
                        <IslandContent>
                            <div className={styles.infoGrid}>
                                <div className={styles.infoItem}>
                                    <Tag>Created</Tag>
                                    <Text>{new Date(project.created_at).toLocaleString()}</Text>
                                </div>
                                <div className={styles.infoItem}>
                                    <Tag>Updated</Tag>
                                    <Text>{new Date(project.updated_at).toLocaleString()}</Text>
                                </div>
                                <div className={styles.infoItem}>
                                    <Tag>Parent</Tag>
                                    <Text>
                                        {project.parent_id && projects[project.parent_id]?.name || 'None'}
                                    </Text>
                                </div>
                                <div className={styles.infoItem}>
                                    <Tag>Subprojects</Tag>
                                    <Text>{children.length}</Text>
                                </div>
                            </div>
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
                                    data={children.map(childId => {
                                        const child = projects[childId];
                                        if (!child) return undefined;
                                        return {
                                            rgItemType: Type.ITEM,
                                            key: child.id,
                                            label: child.name,
                                            description: `${child.repos?.length || 0} repos`,
                                            onClick: () => navigate(`/projects/${child.id}`),
                                            rightNodes: (
                                                <Button
                                                    danger
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setDeleteTarget(child);
                                                    }}
                                                >
                                                    Delete
                                                </Button>
                                            )
                                        };
                                    }).filter((item): item is NonNullable<typeof item> => item !== undefined)}
                                    onSelect={() => {}}
                                    onMouseOut={() => {}}
                                    onScrollToBottom={() => {}}
                                    onResize={() => {}}
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
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setEditingSshKey(key);
                                                    }}
                                                >
                                                    Edit
                                                </Button>
                                                <Button
                                                    danger
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setDeleteSshKeyTarget(key);
                                                    }}
                                                >
                                                    Delete
                                                </Button>
                                            </>
                                        )
                                    }))}
                                    onSelect={() => {}}
                                    onMouseOut={() => {}}
                                    onScrollToBottom={() => {}}
                                    onResize={() => {}}
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
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setEditingRepo(repo);
                                                    }}
                                                >
                                                    Edit
                                                </Button>
                                                <Button
                                                    danger
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setDeleteRepoTarget(repo);
                                                    }}
                                                >
                                                    Delete
                                                </Button>
                                            </>
                                        )
                                    }))}
                                    onSelect={() => {}}
                                    onMouseOut={() => {}}
                                    onScrollToBottom={() => {}}
                                    onResize={() => {}}
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
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>Tasks</Heading>
                        </IslandHeader>
                        <IslandContent>
                            <Text>No tasks yet</Text>
                        </IslandContent>
                    </Island>
                </Tab>
            </Tabs>

            <SubprojectDialog
                show={showAddDialog}
                parentId={actualProjectId!}
                onClose={() => setShowAddDialog(false)}
                onSubmit={handleCreateSubproject}
            />

            <Confirm
                show={!!deleteTarget}
                text="Delete Subproject"
                description={`Are you sure you want to delete "${deleteTarget?.name}"? This action cannot be undone.`}
                confirmLabel="Delete"
                rejectLabel="Cancel"
                onConfirm={handleDeleteSubproject}
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
                onConfirm={handleDeleteSshKey}
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
                onConfirm={handleDeleteRepo}
                onReject={() => setDeleteRepoTarget(null)}
            />
        </div>
    );
};
