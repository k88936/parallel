import type { AlertPayload, AlertSeverity } from '../types';

const WS_BASE_URL = import.meta.env.VITE_API_URL?.replace(/^http/, 'ws') || 'ws://localhost:3000';

type AlertCallback = (alert: AlertPayload) => void;

class AlertWebSocketService {
    private ws: WebSocket | null = null;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 10;
    private reconnectDelay = 1000;
    private callbacks: Set<AlertCallback> = new Set();
    private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
    private shouldReconnect = true;

    connect(): void {
        if (this.ws?.readyState === WebSocket.OPEN) {
            return;
        }

        this.shouldReconnect = true;
        this.ws = new WebSocket(`${WS_BASE_URL}/api/alerts/ws`);

        this.ws.onopen = () => {
            console.log('Alert WebSocket connected');
            this.reconnectAttempts = 0;
            this.reconnectDelay = 1000;
        };

        this.ws.onmessage = (event) => {
            try {
                const payload = JSON.parse(event.data) as AlertPayload;
                this.callbacks.forEach(callback => callback(payload));
            } catch (error) {
                console.error('Failed to parse alert message:', error);
            }
        };

        this.ws.onclose = () => {
            console.log('Alert WebSocket disconnected');
            this.ws = null;
            if (this.shouldReconnect) {
                this.scheduleReconnect();
            }
        };

        this.ws.onerror = (error) => {
            console.error('Alert WebSocket error:', error);
        };
    }

    private scheduleReconnect(): void {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            return;
        }

        this.reconnectTimeout = setTimeout(() => {
            this.reconnectAttempts++;
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
            this.connect();
        }, this.reconnectDelay);
    }

    disconnect(): void {
        this.shouldReconnect = false;
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }

    subscribe(callback: AlertCallback): () => void {
        this.callbacks.add(callback);
        return () => {
            this.callbacks.delete(callback);
        };
    }

    isConnected(): boolean {
        return this.ws?.readyState === WebSocket.OPEN;
    }
}

export const alertWebSocketService = new AlertWebSocketService();

export function getAlertMessage(alert: AlertPayload['alert']): string {
    switch (alert.type) {
        case 'worker_offline':
            return `Worker ${alert.worker_name} went offline${alert.running_tasks.length > 0 ? ` with ${alert.running_tasks.length} running tasks` : ''}`;
        case 'worker_online':
            return `Worker ${alert.worker_name} is now online`;
        case 'task_timeout':
            return `Task "${alert.task_title}" timed out after ${alert.max_execution_time} seconds`;
        case 'task_review_requested':
            return `Task "${alert.task_title}" requires review`;
        case 'task_completed':
            return `Task "${alert.task_title}" completed successfully`;
        case 'task_failed':
            return `Task "${alert.task_title}" failed: ${alert.error}`;
        case 'task_cancelled':
            return `Task "${alert.task_title}" was cancelled`;
        default:
            return 'Unknown alert';
    }
}

export function getSeverityLevel(severity: AlertSeverity): 'info' | 'warning' | 'error' {
    switch (severity) {
        case 'critical':
        case 'error':
            return 'error';
        case 'warning':
            return 'warning';
        default:
            return 'info';
    }
}

export function shouldPlayVoiceAlert(severity: AlertSeverity): boolean {
    return severity === 'critical' || severity === 'error' || severity === 'warning';
}

export function getVoiceAlertMessage(alert: AlertPayload['alert']): string | null {
    switch (alert.type) {
        case 'worker_offline':
            return `Warning: Worker ${alert.worker_name} went offline${alert.running_tasks.length > 0 ? `. ${alert.running_tasks.length} tasks were running.` : ''}`;
        case 'worker_online':
            return `Worker ${alert.worker_name} is back online.`;
        case 'task_timeout':
            return `Error: Task ${alert.task_title} has timed out.`;
        case 'task_review_requested':
            return `Task ${alert.task_title} requires your review.`;
        case 'task_failed':
            return `Error: Task ${alert.task_title} has failed.`;
        case 'task_cancelled':
        case 'task_completed':
            return null;
        default:
            return null;
    }
}
