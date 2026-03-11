import {BrowserRouter, Routes, Route, Navigate} from 'react-router-dom';
import {Provider} from 'react-redux';
import {store} from './store';
import {AppHeader, ProjectLayout} from './components/Layout';
import {ProjectPage, AgentsPage, QueuePage, SettingsPage} from './pages';
import {AlertProvider} from './components/Alerts';
import styles from './App.module.css';
import Theme, {ThemeProvider} from "@jetbrains/ring-ui-built/components/global/theme";

export const App = () => {
    return (
        <Provider store={store}>
            <ThemeProvider theme={Theme.DARK}>
                <AlertProvider>
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
                </AlertProvider>
            </ThemeProvider>
        </Provider>
    );
};
