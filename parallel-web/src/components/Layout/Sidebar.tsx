import {useNavigate} from 'react-router-dom';
import type {Project} from '../../types';
import {ProjectTree} from './ProjectTree';

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
        <aside className="w-[260px] h-full flex flex-col bg-[var(--ring-sidebar-background-color,#1e1e1e)] border-r border-[var(--ring-border-color,#3d3d3d)]">
            <div className="p-4 border-b border-[var(--ring-border-color,#3d3d3d)]">
                <Heading level={3}>Projects</Heading>
            </div>
            <div className="flex-1 overflow-y-auto p-2">
                {loading ? (
                    <div className="flex flex-col items-center justify-center p-4 gap-2">
                        <Loader />
                        <Text>Loading...</Text>
                    </div>
                ) : error && !projects.root ? (
                    <div className="flex flex-col items-center justify-center p-4 gap-2">
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
                    <div className="flex flex-col items-center justify-center p-4 gap-2">
                        <Text>No projects available</Text>
                    </div>
                )}
            </div>
        </aside>
    );
};
