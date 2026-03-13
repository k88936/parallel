/* eslint-disable react-refresh/only-export-components */
import {createContext, useContext, useMemo, useState} from 'react';
import type {Alert} from '../types';
import {categoryToDefaultSound, type AudioSound} from '../services/audioAlerts';

export type AlertType = Alert['type'];

export interface EventAudioConfig {
    enabled: boolean;
    sound: AudioSound;
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
    worker_offline: {enabled: true, sound: 'nope-01.aac', volume: 1.0},
    worker_online: {enabled: true, sound: 'yup-01.aac', volume: 1.0},
    task_timeout: {enabled: true, sound: 'alert-01.aac', volume: 1.0},
    task_review_requested: {enabled: true, sound: 'alert-01.aac', volume: 1.0},
    task_completed: {enabled: true, sound: 'yup-01.aac', volume: 1.0},
    task_failed: {enabled: true, sound: 'nope-01.aac', volume: 1.0},
    task_cancelled: {enabled: true, sound: 'bip-bop-01.aac', volume: 1.0},
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
                const parsed = JSON.parse(storedConfig) as EventAudioConfig & {category?: 'alert' | 'nope' | 'yup' | 'bip-bop' | 'staplebops'};
                if ('sound' in parsed && parsed.sound) {
                    loaded[eventType] = parsed;
                } else if (parsed.category) {
                    loaded[eventType] = {
                        enabled: parsed.enabled,
                        volume: parsed.volume,
                        sound: categoryToDefaultSound(parsed.category),
                    };
                }
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
