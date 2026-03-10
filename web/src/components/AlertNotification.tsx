'use client';

import { useAlerts } from '@/contexts/AlertContext';
import type { AlertWithId, Alert } from '@/types/alert';
import Link from 'next/link';

function getAlertTitle(alert: Alert): string {
    switch (alert.type) {
        case 'worker_offline':
            return `Worker Offline: ${alert.worker_name}`;
        case 'worker_online':
            return `Worker Online: ${alert.worker_name}`;
        case 'task_timeout':
            return `Task Timeout: ${alert.task_title}`;
        case 'task_review_requested':
            return `Review Requested: ${alert.task_title}`;
        case 'task_completed':
            return `Task Completed: ${alert.task_title}`;
        case 'task_failed':
            return `Task Failed: ${alert.task_title}`;
        case 'task_cancelled':
            return `Task Cancelled: ${alert.task_title}`;
        default:
            return 'Alert';
    }
}

function getAlertMessage(alert: Alert): string {
    switch (alert.type) {
        case 'worker_offline':
            return alert.running_tasks.length > 0
                ? `${alert.running_tasks.length} task(s) were re-queued`
                : 'No running tasks';
        case 'worker_online':
            return 'Worker is now connected';
        case 'task_timeout':
            return `Exceeded max execution time of ${alert.max_execution_time}s`;
        case 'task_review_requested':
            return 'Task is waiting for your review';
        case 'task_completed':
            return 'Task completed successfully';
        case 'task_failed':
            return alert.error;
        case 'task_cancelled':
            return 'Task was cancelled';
        default:
            return '';
    }
}

function getAlertLink(alert: Alert): string | null {
    switch (alert.type) {
        case 'task_timeout':
        case 'task_review_requested':
        case 'task_completed':
        case 'task_failed':
        case 'task_cancelled':
            return `/tasks/${alert.task_id}`;
        default:
            return null;
    }
}

const severityStyles = {
    info: 'bg-blue-50 border-blue-200 text-blue-900',
    warning: 'bg-yellow-50 border-yellow-200 text-yellow-900',
    error: 'bg-red-50 border-red-200 text-red-900',
    critical: 'bg-red-100 border-red-300 text-red-900',
};

const severityIcons = {
    info: 'ℹ️',
    warning: '⚠️',
    error: '❌',
    critical: '🚨',
};

interface AlertItemProps {
    alert: AlertWithId;
    onDismiss: (id: string) => void;
}

function AlertItem({ alert, onDismiss }: AlertItemProps) {
    const title = getAlertTitle(alert.alert);
    const message = getAlertMessage(alert.alert);
    const link = getAlertLink(alert.alert);
    const time = new Date(alert.alert.timestamp).toLocaleTimeString();

    const content = (
        <div className={`border rounded-lg p-3 shadow-sm ${severityStyles[alert.severity]}`}>
            <div className="flex items-start gap-2">
                <span className="text-lg">{severityIcons[alert.severity]}</span>
                <div className="flex-1 min-w-0">
                    <div className="font-medium text-sm">{title}</div>
                    <div className="text-xs opacity-75 mt-0.5">{message}</div>
                    <div className="text-xs opacity-50 mt-1">{time}</div>
                </div>
                <button
                    onClick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        onDismiss(alert.id);
                    }}
                    className="text-lg opacity-50 hover:opacity-100"
                >
                    ×
                </button>
            </div>
        </div>
    );

    if (link) {
        return (
            <Link href={link} onClick={() => onDismiss(alert.id)}>
                {content}
            </Link>
        );
    }

    return content;
}

export function AlertNotification() {
    const { alerts, dismissAlert, clearAlerts, isConnected } = useAlerts();

    if (alerts.length === 0) {
        return null;
    }

    return (
        <div className="fixed top-4 right-4 w-80 z-50 space-y-2">
            <div className="flex items-center justify-between px-2 py-1 text-xs text-gray-500">
                <span className="flex items-center gap-1">
                    <span className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
                    {isConnected ? 'Connected' : 'Disconnected'}
                </span>
                {alerts.length > 1 && (
                    <button
                        onClick={clearAlerts}
                        className="hover:text-gray-700"
                    >
                        Clear all ({alerts.length})
                    </button>
                )}
            </div>
            <div className="max-h-96 overflow-y-auto space-y-2">
                {alerts.slice(0, 5).map((alert) => (
                    <AlertItem
                        key={alert.id}
                        alert={alert}
                        onDismiss={dismissAlert}
                    />
                ))}
            </div>
        </div>
    );
}
