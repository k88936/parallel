/* eslint-disable react-refresh/only-export-components */
import {createContext, useContext, useMemo, useState} from 'react';
import type {Alert} from '../types';

export type AlertType = Alert['type'];
export type AudioCategory = 'alert' | 'nope' | 'yup' | 'bip-bop' | 'staplebops';

export interface EventAudioConfig {
    enabled: boolean;
    category: AudioCategory;
    volume: number;
}

export interface AlertSoundSettings {
    worker_offline: EventAudioConfig;
    worker_online: EventAudioConfig;
    task_timeout: EventAudioConfig;
    task_review_requested: EventAudioConfig;
    task_completed: EventAudioConfig;
    task_failed: EventAudioConfig;
    task_cancelled: EventAudioConfig;
}

interface AlertPreferencesContextValue extends AlertSoundSettings {
    setEventConfig: (eventType: AlertType, config: EventAudioConfig) => void;
    getEventConfig: (eventType: AlertType) => EventAudioConfig;
    resetToDefaults: () => void;
}

const DEFAULT_CONFIGS: AlertSoundSettings = {
    worker_offline: {enabled: true, category: 'nope', volume: 1.0},
    worker_online: {enabled: true, category: 'yup', volume: 1.0},
    task_timeout: {enabled: true, category: 'alert', volume: 1.0},
    task_review_requested: {enabled: true, category: 'alert', volume: 1.0},
    task_completed: {enabled: true, category: 'yup', volume: 1.0},
    task_failed: {enabled: true, category: 'nope', volume: 1.0},
    task_cancelled: {enabled: true, category: 'bip-bop', volume: 1.0},
};

const STORAGE_PREFIX = 'parallel.alert.sound.';

const loadSettings = (): AlertSoundSettings => {
    if (typeof window === 'undefined') {
        return DEFAULT_CONFIGS;
    }

    const loaded: AlertSoundSettings = {...DEFAULT_CONFIGS};
    for (const eventType of Object.keys(DEFAULT_CONFIGS) as AlertType[]) {
        const storedConfig = window.localStorage.getItem(`${STORAGE_PREFIX}${eventType}`);
        if (storedConfig) {
            try {
                loaded[eventType] = JSON.parse(storedConfig);
            } catch {
                // Keep default if parsing fails
            }
        }
    }
    return loaded;
};

const saveEventConfig = (eventType: AlertType, config: EventAudioConfig) => {
    if (typeof window === 'undefined') return;
    window.localStorage.setItem(`${STORAGE_PREFIX}${eventType}`, JSON.stringify(config));
};

const AlertPreferencesContext = createContext<AlertPreferencesContextValue | undefined>(undefined);

export const AlertPreferencesProvider = ({children}: {children: React.ReactNode}) => {
    const [settings, setSettings] = useState<AlertSoundSettings>(loadSettings);

    const setEventConfig = (eventType: AlertType, config: EventAudioConfig) => {
        setSettings(prev => {
            const updated = {...prev, [eventType]: config};
            saveEventConfig(eventType, config);
            return updated;
        });
    };

    const getEventConfig = (eventType: AlertType): EventAudioConfig => {
        return settings[eventType];
    };

    const resetToDefaults = () => {
        setSettings(DEFAULT_CONFIGS);
        for (const eventType of Object.keys(DEFAULT_CONFIGS) as AlertType[]) {
            saveEventConfig(eventType, DEFAULT_CONFIGS[eventType]);
        }
    };

    const value = useMemo(() => ({
        ...settings,
        setEventConfig,
        getEventConfig,
        resetToDefaults,
    }), [settings]);

    return <AlertPreferencesContext.Provider value={value}>{children}</AlertPreferencesContext.Provider>;
};

export const useAlertPreferences = () => {
    const context = useContext(AlertPreferencesContext);
    if (!context) {
        throw new Error('useAlertPreferences must be used within AlertPreferencesProvider');
    }

    return context;
};
