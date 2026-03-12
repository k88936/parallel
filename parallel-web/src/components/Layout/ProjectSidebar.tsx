import {useNavigate} from 'react-router-dom';
import type {Project} from '../../types';
import {ProjectTree} from './ProjectTree';
import {Sidebar} from './Sidebar';

import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';

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
                    <div className="flex flex-col items-center justify-center p-4 gap-2">
                        <Loader/>
                        <Text>Loading...</Text>
                    </div>
                ) : error && !projects.root ? (
                    <div className="flex flex-col items-center justify-center p-4 gap-2">
                        <Text>{error}</Text>
                    </div>
                ) : projects.root ? (
                    <div className="p-2">
                        <ProjectTree
                            projectId="root"
                            projects={projects}
                            childrenByParent={childrenByParent}
                            expandedNodes={expandedNodes}
                            selectedId={selectedProjectId}
                            onNodeClick={handleNodeClick}
                            onNodeToggle={handleNodeToggle}
                        />
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center p-4 gap-2">
                        <Text>No projects available</Text>
                    </div>
                )}
        </Sidebar>
    );
};
