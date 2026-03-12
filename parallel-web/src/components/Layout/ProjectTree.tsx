import type {Project} from '../../types';
import styles from './Sidebar.module.css';

import Button from '@jetbrains/ring-ui-built/components/button/button';
import Icon from '@jetbrains/ring-ui-built/components/icon/icon';
import {Color} from '@jetbrains/ring-ui-built/components/icon/icon.constants';

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
        <div className={styles.treeNode}>
            <div className={`${styles.nodeContent} ${isSelected ? styles.selected : ''}`}>
                <Button
                    inline
                    className={`${styles.chevron} ${isExpanded ? styles.expanded : ''}`}
                    onClick={(e) => {
                        e.stopPropagation();
                        void onNodeToggle(projectId);
                    }}
                    icon={chevronRightIcon}
                />
                <Icon
                    glyph={folderIcon}
                    color={isSelected ? Color.BLUE : Color.DEFAULT}
                    className={styles.nodeIcon}
                />
                <Button
                    inline
                    className={styles.nodeButton}
                    onClick={() => onNodeClick(projectId)}
                    active={isSelected}
                >
                    {project.name}
                </Button>
            </div>
            {isExpanded && hasChildren && (
                <div className={styles.children}>
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
                </div>
            )}
        </div>
    );
};
