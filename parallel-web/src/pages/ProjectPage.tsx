import {useEffect, useState} from 'react';
import {useNavigate, useParams} from 'react-router-dom';
import {useAppDispatch, useAppSelector} from '../store/hooks';
import {selectProject, fetchProjectChildren, fetchRootProject} from '../store/slices/projectsSlice';
import type {Project} from '../types';
import styles from './ProjectPage.module.css';

import Breadcrumbs from '@jetbrains/ring-ui-built/components/breadcrumbs/breadcrumbs';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import ButtonGroup from '@jetbrains/ring-ui-built/components/button-group/button-group';
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

type TabId = 'overview' | 'settings' | 'repos' | 'tasks';

function setShowForm(_b: boolean) {
}

export const ProjectPage = () => {
    const {projectId} = useParams<{ projectId: string }>();
    const navigate = useNavigate();
    const dispatch = useAppDispatch();
    const {projects, childrenByParent, rootProjectId, loading} = useAppSelector((state) => state.projects);
    const [activeTab, setActiveTab] = useState<TabId>('overview');

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

    const subprojectListData = children.map(childId => {
        const child = projects[childId];
        if (!child) return undefined;
        return {
            rgItemType: Type.ITEM,
            key: child.id,
            label: child.name,
            description: `${child.repos?.length || 0} repos`,
            onClick: () => navigate(`/projects/${child.id}`)
        };
    }).filter((item): item is NonNullable<typeof item> => item !== undefined);

    const repoListData = project.repos.map((repo, i) => ({
        rgItemType: Type.ITEM,
        key: i.toString(),
        label: repo.name,
        description: repo.url
    }));

    const sshKeyListData = project.ssh_keys.map((key, i) => ({
        rgItemType: Type.ITEM,
        key: i.toString(),
        label: key.name,
        description: `${key.key.substring(0, 30)}...`
    }));

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
                <ButtonGroup>
                    <Button primary onClick={() => setShowForm(true)}>
                        Add Subproject
                    </Button>
                    <Button onClick={() => setShowForm(true)}>
                        Edit
                    </Button>
                </ButtonGroup>
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

                    {children.length > 0 && (
                        <Island>
                            <IslandHeader border>
                                <Heading level={3}>Subprojects</Heading>
                            </IslandHeader>
                            <IslandContent>
                                <List
                                    data={subprojectListData}
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
                            </IslandContent>
                        </Island>
                    )}
                </Tab>
                <Tab id="settings" title="Settings">
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>SSH Keys</Heading>
                        </IslandHeader>
                        <IslandContent>
                            {project.ssh_keys.length === 0 ? (
                                <Text>No SSH keys configured</Text>
                            ) : (
                                <List
                                    data={sshKeyListData}
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
                            <div className={styles.actions}>
                                <Button onClick={() => setShowForm(true)}>Edit Settings</Button>
                            </div>
                        </IslandContent>
                    </Island>
                </Tab>
                <Tab id="repos" title="Repositories">
                    <Island>
                        <IslandHeader border>
                            <Heading level={3}>Repositories</Heading>
                        </IslandHeader>
                        <IslandContent>
                            {project.repos.length === 0 ? (
                                <Text>No repositories configured</Text>
                            ) : (
                                <List
                                    data={repoListData}
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
        </div>
    );
};
