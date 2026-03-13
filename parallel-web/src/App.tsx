import {BrowserRouter, Navigate, Route, Routes} from 'react-router-dom';
import {AppHeader, ProjectLayout} from './components/Layout';
import {AlertProvider} from './components/Alerts';
import {AgentsPage, ProjectPage, QueuePage, SettingsPage} from './pages';
import Theme, {ThemeProvider} from '@jetbrains/ring-ui-built/components/global/theme';
import Group from "@jetbrains/ring-ui-built/components/group/group.js";

export const App = () => {
    return (
        <ThemeProvider theme={Theme.LIGHT}>
            <AlertProvider>
                <BrowserRouter>
                    <Group className="flex flex-row min-h-screen">
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
                    </Group>
                </BrowserRouter>
            </AlertProvider>
        </ThemeProvider>
    );
};
