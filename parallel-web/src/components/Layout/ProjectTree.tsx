import type {Project} from '../../types';

import Button from '@jetbrains/ring-ui-built/components/button/button';
import Icon from '@jetbrains/ring-ui-built/components/icon/icon';
import {Color} from '@jetbrains/ring-ui-built/components/icon/icon.constants';
import Group from '@jetbrains/ring-ui-built/components/group/group';

import folderIcon from '@jetbrains/icons/folder';
import chevronRightIcon from '@jetbrains/icons/chevron-right';

interface ProjectTreeProps {
    projectId: string;
    projects: Record<string, Project>;
    childrenByParent: Record<string, string[]>;
    expandedNodes: string[];
    selectedId: string | null;
    onNodeClick: (id: string) => void;
    onNodeToggle: (id: string) => void | Promise<void>;
}

export const ProjectTree = ({
    projectId,
    projects,
    childrenByParent,
    expandedNodes,
    selectedId,
    onNodeClick,
    onNodeToggle,
}: ProjectTreeProps) => {
    const project = projects[projectId];
    const children = childrenByParent[projectId] || [];
    const isExpanded = expandedNodes.includes(projectId);
    const isSelected = selectedId === projectId;

    if (!project) return null;

    const hasChildren = children.length > 0;

    return (
        <Group className="select-none">
            <Group className={`flex items-center px-2 py-1 rounded gap-0.5 ${isSelected ? '' : ''}`}>
                <Button
                    inline
                    className={`w-5 min-w-[20px] !p-0 transition-transform duration-150 ${isExpanded ? 'rotate-90' : ''}`}
                    onClick={(e) => {
                        e.stopPropagation();
                        void onNodeToggle(projectId);
                    }}
                    icon={chevronRightIcon}
                />
                <Icon
                    glyph={folderIcon}
                    color={isSelected ? Color.BLUE : Color.DEFAULT}
                    className="shrink-0"
                />
                <Button
                    inline
                    className="flex-1 justify-start text-left overflow-hidden text-ellipsis"
                    onClick={() => onNodeClick(projectId)}
                    active={isSelected}
                >
                    {project.name}
                </Button>
            </Group>
            {isExpanded && hasChildren && (
                <Group className="pl-4">
                    {children.map((childId) => (
                        <ProjectTree
                            key={childId}
                            projectId={childId}
                            projects={projects}
                            childrenByParent={childrenByParent}
                            expandedNodes={expandedNodes}
                            selectedId={selectedId}
                            onNodeClick={onNodeClick}
                            onNodeToggle={onNodeToggle}
                        />
                    ))}
                </Group>
            )}
        </Group>
    );
};
