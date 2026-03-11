import {Outlet} from 'react-router-dom';
import {Sidebar} from './Sidebar';
import styles from './ProjectLayout.module.css';

export const ProjectLayout = () => {
    return (
        <div className={styles.layout}>
            <Sidebar/>
            <main className={styles.main}>
                <Outlet/>
            </main>
        </div>
    );
};
