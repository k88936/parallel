import {useAppSelector} from '../../store/hooks';
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
    selectedId: string | null;
    onNodeClick: (id: string) => void;
    onNodeToggle: (id: string) => void;
    onLoadChildren: (id: string) => void;
}

export const ProjectTree = ({
    projectId,
    projects,
    selectedId,
    onNodeClick,
    onNodeToggle,
    onLoadChildren,
}: ProjectTreeProps) => {
    const {childrenByParent, expandedNodes} = useAppSelector((state) => state.projects);
    const project = projects[projectId];
    const children = childrenByParent[projectId] || [];
    const isExpanded = expandedNodes.includes(projectId);
    const isSelected = selectedId === projectId;

    if (!project) return null;

    const hasChildren = children.length > 0;
    const handleExpand = () => {
        onNodeToggle(projectId);
        if (!isExpanded && !hasChildren) {
            onLoadChildren(projectId);
        }
    };

    return (
        <div className={styles.treeNode}>
            <div className={`${styles.nodeContent} ${isSelected ? styles.selected : ''}`}>
                <Button
                    inline
                    className={`${styles.chevron} ${isExpanded ? styles.expanded : ''}`}
                    onClick={(e) => {
                        e.stopPropagation();
                        handleExpand();
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
                            selectedId={selectedId}
                            onNodeClick={onNodeClick}
                            onNodeToggle={onNodeToggle}
                            onLoadChildren={onLoadChildren}
                        />
                    ))}
                </div>
            )}
        </div>
    );
};
