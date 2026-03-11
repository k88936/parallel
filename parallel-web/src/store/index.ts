import { configureStore } from '@reduxjs/toolkit';
import projectsReducer from './slices/projectsSlice';
import workersReducer from './slices/workersSlice';
import tasksReducer from './slices/tasksSlice';

export const store = configureStore({
    reducer: {
        projects: projectsReducer,
        workers: workersReducer,
        tasks: tasksReducer,
    },
});

export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
