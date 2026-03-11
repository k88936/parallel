import {useEffect} from 'react';
import {useNavigate} from 'react-router-dom';
import {useAppDispatch, useAppSelector} from '../../store/hooks';
import {fetchRootProject, fetchProjectChildren, toggleNode, selectProject} from '../../store/slices/projectsSlice';
import {ProjectTree} from './ProjectTree';
import styles from './Sidebar.module.css';

import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';

export const Sidebar = () => {
    const dispatch = useAppDispatch();
    const navigate = useNavigate();
    const {rootProjectId, projects, selectedProjectId, loading} = useAppSelector(
        (state) => state.projects
    );

    useEffect(() => {
        dispatch(fetchRootProject());
    }, [dispatch]);

    const handleNodeClick = (projectId: string) => {
        dispatch(selectProject(projectId));
        navigate(`/projects/${projectId}`);
    };

    const handleNodeToggle = (projectId: string) => {
        dispatch(toggleNode(projectId));
    };

    const handleLoadChildren = (projectId: string) => {
        dispatch(fetchProjectChildren(projectId));
    };

    return (
        <aside className={styles.sidebar}>
            <div className={styles.header}>
                <Heading level={3}>Projects</Heading>
            </div>
            <div className={styles.content}>
                {loading && !rootProjectId ? (
                    <div className={styles.loading}>
                        <Loader/>
                        <Text>Loading...</Text>
                    </div>
                ) : rootProjectId ? (
                    <ProjectTree
                        projectId={rootProjectId}
                        projects={projects}
                        selectedId={selectedProjectId}
                        onNodeClick={handleNodeClick}
                        onNodeToggle={handleNodeToggle}
                        onLoadChildren={handleLoadChildren}
                    />
                ) : null}
            </div>
        </aside>
    );
};
