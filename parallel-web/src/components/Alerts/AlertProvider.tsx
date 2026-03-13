import {useCallback, useEffect} from 'react';
import alertService from '@jetbrains/ring-ui-built/components/alert-service/alert-service';
import type {AlertPayload} from '../../types';
import {AlertPreferencesProvider, useAlertPreferences} from '../../contexts/AlertContext';
import {
    alertWebSocketService,
    getAlertMessage,
    getSeverityLevel,
    shouldPlayVoiceAlert,
} from '../../services/alertService';
import {getRandomAudioFile, getAudioPath, playAudioFile} from '../../services/audioAlerts';

const AlertListener = ({children}: {children: React.ReactNode}) => {
    const {getEventConfig} = useAlertPreferences();

    const playSound = useCallback((alertType: string, volume: number) => {
        const eventConfig = getEventConfig(alertType as any);
        if (!eventConfig.enabled) return;

        const audioFile = getRandomAudioFile(eventConfig.category);
        const audioPath = getAudioPath(audioFile);
        playAudioFile(audioPath, eventConfig.volume * volume).catch(err => {
            console.error('Failed to play alert sound:', err);
        });
    }, [getEventConfig]);

    const handleAlert = useCallback((payload: AlertPayload) => {
        const message = getAlertMessage(payload.alert);
        const severity = getSeverityLevel(payload.severity);

        switch (severity) {
            case 'error':
                alertService.error(message, 5000);
                break;
            case 'warning':
                alertService.warning(message, 5000);
                break;
            default:
                alertService.message(message, 5000);
        }

        if (shouldPlayVoiceAlert(payload.severity)) {
            playSound(payload.alert.type, 1.0);
        }
    }, [playSound]);

    useEffect(() => {
        alertWebSocketService.connect();
        const unsubscribe = alertWebSocketService.subscribe(handleAlert);

        return () => {
            unsubscribe();
            alertWebSocketService.disconnect();
        };
    }, [handleAlert]);

    return <>{children}</>;
};

export function AlertProvider({children}: {children: React.ReactNode}) {
    return (
        <AlertPreferencesProvider>
            <AlertListener>{children}</AlertListener>
        </AlertPreferencesProvider>
    );
}
