/* eslint-disable react-refresh/only-export-components */
import {createContext, useContext, useEffect, useMemo, useState} from 'react';

interface AlertPreferencesContextValue {
    voiceEnabled: boolean;
    setVoiceEnabled: (enabled: boolean) => void;
}

const STORAGE_KEY = 'parallel.voiceEnabled';

const AlertPreferencesContext = createContext<AlertPreferencesContextValue | undefined>(undefined);

export const AlertPreferencesProvider = ({children}: {children: React.ReactNode}) => {
    const [voiceEnabled, setVoiceEnabled] = useState<boolean>(() => {
        if (typeof window === 'undefined') {
            return true;
        }

        const storedValue = window.localStorage.getItem(STORAGE_KEY);
        if (storedValue === null) {
            return true;
        }

        return storedValue === 'true';
    });

    useEffect(() => {
        if (typeof window !== 'undefined') {
            window.localStorage.setItem(STORAGE_KEY, String(voiceEnabled));
        }
    }, [voiceEnabled]);

    const value = useMemo(() => ({
        voiceEnabled,
        setVoiceEnabled,
    }), [voiceEnabled]);

    return <AlertPreferencesContext.Provider value={value}>{children}</AlertPreferencesContext.Provider>;
};

export const useAlertPreferences = () => {
    const context = useContext(AlertPreferencesContext);
    if (!context) {
        throw new Error('useAlertPreferences must be used within AlertPreferencesProvider');
    }

    return context;
};
