import {BrowserRouter, Routes, Route, Navigate} from 'react-router-dom';
import {Provider} from 'react-redux';
import {store} from './store';
import {AppHeader, ProjectLayout} from './components/Layout';
import {ProjectPage, AgentsPage, QueuePage, SettingsPage} from './pages';
import styles from './App.module.css';
import Theme, {ThemeProvider} from "@jetbrains/ring-ui-built/components/global/theme";

export const App = () => {
    return (
        <Provider store={store}>
            <ThemeProvider theme={Theme.DARK}>
                <BrowserRouter>
                    <div className={styles.app}>
                        <AppHeader/>
                        <Routes>
                            <Route path="/" element={<Navigate to="/projects/root" replace/>}/>
                            <Route path="/projects" element={<ProjectLayout/>}>
                                <Route index element={<Navigate to="root" replace/>}/>
                                <Route path=":projectId" element={<ProjectPage/>}/>
                            </Route>
                            <Route path="/agents" element={<AgentsPage/>}/>
                            <Route path="/queue" element={<QueuePage/>}/>
                            <Route path="/settings" element={<SettingsPage/>}/>
                        </Routes>
                    </div>
                </BrowserRouter>
            </ThemeProvider>
        </Provider>
    );
};
