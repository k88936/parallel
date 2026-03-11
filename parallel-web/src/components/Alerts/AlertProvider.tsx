import { useEffect, useCallback, useRef } from 'react';
import { useDispatch } from 'react-redux';
import alertService from '@jetbrains/ring-ui-built/components/alert-service/alert-service';
import type { AlertPayload } from '../../types';
import { useAppSelector } from '../../store/hooks';
import {
    alertWebSocketService,
    getAlertMessage,
    getSeverityLevel,
    getVoiceAlertMessage,
} from '../../services/alertService';
import { addAlert } from '../../store/slices/alertsSlice';

export function AlertProvider({ children }: { children: React.ReactNode }) {
    const dispatch = useDispatch();
    const { voiceEnabled } = useAppSelector((state) => state.alerts);
    const speechSynthRef = useRef<SpeechSynthesis | null>(null);

    const speak = useCallback((text: string) => {
        if (!voiceEnabled || !text) return;
        
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
        dispatch(addAlert(payload));
        
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
        
        const voiceMessage = getVoiceAlertMessage(payload.alert);
        if (voiceMessage) {
            speak(voiceMessage);
        }
    }, [dispatch, speak]);

    useEffect(() => {
        alertWebSocketService.connect();
        const unsubscribe = alertWebSocketService.subscribe(handleAlert);
        
        return () => {
            unsubscribe();
            alertWebSocketService.disconnect();
        };
    }, [handleAlert]);

    return <>{children}</>;
}
