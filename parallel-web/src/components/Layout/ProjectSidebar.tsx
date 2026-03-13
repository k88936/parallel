import {useNavigate} from 'react-router-dom';
import type {Project} from '../../types';
import {ProjectTree} from './ProjectTree';
import {Sidebar} from './Sidebar';

import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Group from '@jetbrains/ring-ui-built/components/group/group';

interface ProjectSidebarProps {
    projects: Record<string, Project>;
    childrenByParent: Record<string, string[]>;
    selectedProjectId: string | null;
    expandedNodes: string[];
    loading: boolean;
    error: string | null;
    onNodeToggle: (projectId: string) => void;
    onLoadChildren: (projectId: string) => Promise<void>;
}

export const ProjectSidebar = ({
                                    projects,
                                    childrenByParent,
                                   selectedProjectId,
                                   expandedNodes,
                                    loading,
                                    error,
                                    onNodeToggle,
                                    onLoadChildren,
                                }: ProjectSidebarProps) => {
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
        <Sidebar title="Projects">
                {loading ? (
                    <Group className="flex flex-col items-center justify-center p-4 gap-2">
                        <Loader/>
                        <Text>Loading...</Text>
                    </Group>
                ) : error && !projects.root ? (
                    <Group className="flex flex-col items-center justify-center p-4 gap-2">
                        <Text>{error}</Text>
                    </Group>
                ) : projects.root ? (
                    <Group className="p-2">
                        <ProjectTree
                            projectId="root"
                            projects={projects}
                            childrenByParent={childrenByParent}
                            expandedNodes={expandedNodes}
                            selectedId={selectedProjectId}
                            onNodeClick={handleNodeClick}
                            onNodeToggle={handleNodeToggle}
                        />
                    </Group>
                ) : (
                    <Group className="flex flex-col items-center justify-center p-4 gap-2">
                        <Text>No projects available</Text>
                    </Group>
                )}
        </Sidebar>
    );
};
