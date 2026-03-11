import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import type { AlertPayload, AlertSeverity } from '../../types';

interface AlertState {
    alerts: AlertPayload[];
    maxAlerts: number;
    voiceEnabled: boolean;
    unreadCount: number;
    lastAlert: AlertPayload | null;
}

const initialState: AlertState = {
    alerts: [],
    maxAlerts: 100,
    voiceEnabled: true,
    unreadCount: 0,
    lastAlert: null,
};

const alertsSlice = createSlice({
    name: 'alerts',
    initialState,
    reducers: {
        addAlert: (state, action: PayloadAction<AlertPayload>) => {
            state.alerts.unshift(action.payload);
            if (state.alerts.length > state.maxAlerts) {
                state.alerts.pop();
            }
            state.lastAlert = action.payload;
            const criticalSeverities: AlertSeverity[] = ['error', 'critical'];
            if (criticalSeverities.includes(action.payload.severity)) {
                state.unreadCount++;
            }
        },
        clearAlerts: (state) => {
            state.alerts = [];
            state.unreadCount = 0;
        },
        dismissAlert: (state, action: PayloadAction<number>) => {
            state.alerts.splice(action.payload, 1);
        },
        markAllRead: (state) => {
            state.unreadCount = 0;
        },
        setVoiceEnabled: (state, action: PayloadAction<boolean>) => {
            state.voiceEnabled = action.payload;
        },
        toggleVoice: (state) => {
            state.voiceEnabled = !state.voiceEnabled;
        },
    },
});

export const {
    addAlert,
    clearAlerts,
    dismissAlert,
    markAllRead,
    setVoiceEnabled,
    toggleVoice,
} = alertsSlice.actions;

export default alertsSlice.reducer;
