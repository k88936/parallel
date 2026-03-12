import {BrowserRouter, Navigate, Route, Routes} from 'react-router-dom';
import {AppHeader, ProjectLayout} from './components/Layout';
import {AlertProvider} from './components/Alerts';
import {AgentsPage, ProjectPage, QueuePage, SettingsPage} from './pages';
import Theme, {ThemeProvider} from '@jetbrains/ring-ui-built/components/global/theme';

export const App = () => {
    return (
        <ThemeProvider theme={Theme.DARK}>
            <AlertProvider>
                <BrowserRouter>
                    <div className="flex flex-row min-h-screen bg-[var(--ring-content-background-color,#1e1e1e)] text-[var(--ring-text-color,#fff)]">
                        <AppHeader />
                        <Routes>
                            <Route path="/" element={<Navigate to="/projects/root" replace />} />
                            <Route path="/projects" element={<ProjectLayout />}>
                                <Route index element={<Navigate to="root" replace />} />
                                <Route path=":projectId" element={<ProjectPage />} />
                            </Route>
                            <Route path="/agents" element={<AgentsPage />} />
                            <Route path="/queue" element={<QueuePage />} />
                            <Route path="/settings" element={<SettingsPage />} />
                        </Routes>
                    </div>
                </BrowserRouter>
            </AlertProvider>
        </ThemeProvider>
    );
};
