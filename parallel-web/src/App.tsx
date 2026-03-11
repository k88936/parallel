import {BrowserRouter, Routes, Route, Navigate} from 'react-router-dom';
import {Provider} from 'react-redux';
import {store} from './store';
import {Sidebar} from './components/Layout';
import {ProjectDetail} from './pages';
import styles from './App.module.css';
import Theme, {ThemeProvider} from "@jetbrains/ring-ui-built/components/global/theme";

export const App = () => {
    return (
        <Provider store={store}>
            <ThemeProvider theme={Theme.DARK}>
                <BrowserRouter>
                    <div className={styles.app}>
                        <Sidebar/>
                        <main className={styles.main}>
                            <Routes>
                                <Route path="/" element={<Navigate to="/projects/root" replace/>}/>
                                <Route path="/projects/:projectId" element={<ProjectDetail/>}/>
                            </Routes>
                        </main>
                    </div>
                </BrowserRouter>
            </ThemeProvider>
        </Provider>
    );
};
