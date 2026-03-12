import {useNavigate} from 'react-router-dom';
import type {Project} from '../../types';
import {ProjectTree} from './ProjectTree';
import styles from './Sidebar.module.css';

import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';

interface SidebarProps {
    projects: Record<string, Project>;
    childrenByParent: Record<string, string[]>;
    selectedProjectId: string | null;
    expandedNodes: string[];
    loading: boolean;
    error: string | null;
    onNodeToggle: (projectId: string) => void;
    onLoadChildren: (projectId: string) => Promise<void>;
}

export const Sidebar = ({
    projects,
    childrenByParent,
    selectedProjectId,
    expandedNodes,
    loading,
    error,
    onNodeToggle,
    onLoadChildren,
}: SidebarProps) => {
    const navigate = useNavigate();

    const handleNodeClick = (projectId: string) => {
        navigate(`/projects/${projectId}`);
    };

    const handleNodeToggle = async (projectId: string) => {
        const isExpanded = expandedNodes.includes(projectId);
        onNodeToggle(projectId);
        if (!isExpanded) {
            await onLoadChildren(projectId);
        }
    };

    return (
        <aside className={styles.sidebar}>
            <div className={styles.header}>
                <Heading level={3}>Projects</Heading>
            </div>
            <div className={styles.content}>
                {loading ? (
                    <div className={styles.loading}>
                        <Loader />
                        <Text>Loading...</Text>
                    </div>
                ) : error && !projects.root ? (
                    <div className={styles.loading}>
                        <Text>{error}</Text>
                    </div>
                ) : projects.root ? (
                    <ProjectTree
                        projectId="root"
                        projects={projects}
                        childrenByParent={childrenByParent}
                        expandedNodes={expandedNodes}
                        selectedId={selectedProjectId}
                        onNodeClick={handleNodeClick}
                        onNodeToggle={handleNodeToggle}
                    />
                ) : (
                    <div className={styles.loading}>
                        <Text>No projects available</Text>
                    </div>
                )}
            </div>
        </aside>
    );
};
