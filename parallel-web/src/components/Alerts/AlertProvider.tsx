import {useCallback, useEffect, useRef} from 'react';
import alertService from '@jetbrains/ring-ui-built/components/alert-service/alert-service';
import type {AlertPayload} from '../../types';
import {AlertPreferencesProvider, useAlertPreferences} from '../../contexts/AlertContext';
import {
    alertWebSocketService,
    getAlertMessage,
    getSeverityLevel,
    getVoiceAlertMessage,
    shouldPlayVoiceAlert,
} from '../../services/alertService';

const AlertListener = ({children}: {children: React.ReactNode}) => {
    const {voiceEnabled} = useAlertPreferences();
    const speechSynthRef = useRef<SpeechSynthesis | null>(null);

    const speak = useCallback((text: string) => {
        if (!voiceEnabled || !text || typeof window === 'undefined' || !('speechSynthesis' in window)) {
            return;
        }

        if (!speechSynthRef.current) {
            speechSynthRef.current = window.speechSynthesis;
        }

        const utterance = new SpeechSynthesisUtterance(text);
        utterance.rate = 1.0;
        utterance.pitch = 1.0;
        utterance.volume = 1.0;
        speechSynthRef.current.speak(utterance);
    }, [voiceEnabled]);

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

        if (voiceEnabled && shouldPlayVoiceAlert(payload.severity)) {
            const voiceMessage = getVoiceAlertMessage(payload.alert);
            if (voiceMessage) {
                speak(voiceMessage);
            }
        }
    }, [speak, voiceEnabled]);

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
