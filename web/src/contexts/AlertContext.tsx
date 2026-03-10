'use client';

import React, { createContext, useContext, useEffect, useState, useCallback, useRef } from 'react';
import type { AlertPayload, AlertWithId, AlertSeverity } from '@/types/alert';

const MAX_ALERTS = 50;
const ALERT_TIMEOUT_MS = 10000;

interface AlertContextValue {
    alerts: AlertWithId[];
    dismissAlert: (id: string) => void;
    clearAlerts: () => void;
    isConnected: boolean;
}

const AlertContext = createContext<AlertContextValue | null>(null);

export function useAlerts(): AlertContextValue {
    const context = useContext(AlertContext);
    if (!context) {
        throw new Error('useAlerts must be used within an AlertProvider');
    }
    return context;
}

interface AlertProviderProps {
    children: React.ReactNode;
}

export function AlertProvider({ children }: AlertProviderProps) {
    const [alerts, setAlerts] = useState<AlertWithId[]>([]);
    const [isConnected, setIsConnected] = useState(false);
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const alertIdRef = useRef(0);

    const addAlert = useCallback((payload: AlertPayload) => {
        const id = `alert-${++alertIdRef.current}`;
        const alertWithId: AlertWithId = { ...payload, id };
        
        setAlerts(prev => {
            const newAlerts = [alertWithId, ...prev].slice(0, MAX_ALERTS);
            return newAlerts;
        });

        if (payload.severity !== 'critical') {
            setTimeout(() => {
                dismissAlert(id);
            }, ALERT_TIMEOUT_MS);
        }
    }, []);

    const dismissAlert = useCallback((id: string) => {
        setAlerts(prev => prev.filter(a => a.id !== id));
    }, []);

    const clearAlerts = useCallback(() => {
        setAlerts([]);
    }, []);

    const connect = useCallback(() => {
        if (wsRef.current?.readyState === WebSocket.OPEN) return;

        const wsUrl = process.env.NEXT_PUBLIC_API_URL 
            ? `${process.env.NEXT_PUBLIC_API_URL.replace('http', 'ws')}/api/alerts/ws`
            : 'ws://localhost:3000/api/alerts/ws';

        const ws = new WebSocket(wsUrl);
        wsRef.current = ws;

        ws.onopen = () => {
            setIsConnected(true);
        };

        ws.onmessage = (event) => {
            try {
                const payload: AlertPayload = JSON.parse(event.data);
                addAlert(payload);
            } catch (e) {
                console.error('Failed to parse alert:', e);
            }
        };

        ws.onclose = () => {
            setIsConnected(false);
            wsRef.current = null;
            reconnectTimeoutRef.current = setTimeout(() => {
                connect();
            }, 3000);
        };

        ws.onerror = () => {
            ws.close();
        };
    }, [addAlert]);

    useEffect(() => {
        connect();

        return () => {
            if (wsRef.current) {
                wsRef.current.close();
            }
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
        };
    }, [connect]);

    return (
        <AlertContext.Provider value={{ alerts, dismissAlert, clearAlerts, isConnected }}>
            {children}
        </AlertContext.Provider>
    );
}
