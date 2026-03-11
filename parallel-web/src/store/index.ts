import { configureStore } from '@reduxjs/toolkit';
import projectsReducer from './slices/projectsSlice';
import workersReducer from './slices/workersSlice';
import tasksReducer from './slices/tasksSlice';
import alertsReducer from './slices/alertsSlice';

export const store = configureStore({
    reducer: {
        projects: projectsReducer,
        workers: workersReducer,
        tasks: tasksReducer,
        alerts: alertsReducer,
    },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
