'use client';

import {useEffect, useState} from 'react';
import {useRouter} from 'next/navigation';
import type {Task, TaskStatus} from '@/types/task';
import {api} from '@/lib/api';

const STATUS_COLORS: Record<TaskStatus, string> = {
    created: 'bg-gray-100 text-gray-800',
    queued: 'bg-blue-100 text-blue-800',
    claimed: 'bg-yellow-100 text-yellow-800',
    in_progress: 'bg-orange-100 text-orange-800',
    awaiting_review: 'bg-purple-100 text-purple-800',
    pending_rework: 'bg-indigo-100 text-indigo-800',
    completed: 'bg-green-100 text-green-800',
    cancelled: 'bg-red-100 text-red-800',
    failed: 'bg-red-100 text-red-800',
};

const STATUS_LABELS: Record<string, string> = {
    created: 'Created',
    queued: 'Queued',
    claimed: 'Claimed',
    in_progress: 'In Progress',
    awaiting_review: 'Awaiting Review',
    pending_rework: 'Pending Rework',
    completed: 'Completed',
    cancelled: 'Cancelled',
    failed: 'Failed',
};

export function TaskList({
                             refreshKey,
                         }: {
    refreshKey: number;
}) {
    const router = useRouter();
    const [tasks, setTasks] = useState<Task[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [statusFilter, setStatusFilter] = useState<TaskStatus | ''>('');

    const fetchTasks = async () => {
        setLoading(true);
        setError(null);
        try {
            const response = await api.listTasks(
                statusFilter ? {status: statusFilter, limit: 100} : {limit: 100}
            );
            setTasks(response.tasks);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to fetch tasks');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchTasks();
    }, [refreshKey, statusFilter]);

    const handleCancel = async (taskId: string, e: React.MouseEvent) => {
        e.stopPropagation();
        if (!confirm('Are you sure you want to cancel this task?')) return;

        try {
            await api.cancelTask(taskId);
            fetchTasks();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to cancel task');
        }
    };

    if (loading) return <div className="text-center py-8">Loading tasks...</div>;
    if (error)
        return <div className="text-center py-8 text-red-600">Error: {error}</div>;

    return (
        <div>
            <div className="mb-4 flex items-center gap-4">
                <h2 className="text-xl font-bold">Tasks</h2>
                <select
                    value={statusFilter}
                    onChange={(e) => setStatusFilter(e.target.value as TaskStatus | '')}
                    className="border rounded px-3 py-1"
                >
                    <option value="">All Statuses</option>
                    {Object.entries(STATUS_LABELS).map(([value, label]) => (
                        <option key={value} value={value}>
                            {label}
                        </option>
                    ))}
                </select>
            </div>

            {tasks.length === 0 ? (
                <div className="text-center py-8 text-gray-500">No tasks found</div>
            ) : (
                <div className="space-y-2">
                    {tasks.map((task) => (
                        <div
                            key={task.id}
                            onClick={() => router.push(`/tasks/${task.id}`)}
                            className="border rounded-lg p-4 hover:bg-gray-50 cursor-pointer transition-colors"
                        >
                            <div className="flex items-start justify-between gap-4">
                                <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2 mb-2">
                    <span
                        className={`px-2 py-1 rounded text-xs font-medium ${
                            STATUS_COLORS[task.status]
                        }`}
                    >
                      {STATUS_LABELS[task.status]}
                    </span>
                                        <span className="text-xs text-gray-500">
                      Priority: {task.priority}
                    </span>
                                        {task.status === 'awaiting_review' && (
                                            <span className="text-xs bg-purple-200 text-purple-800 px-2 py-0.5 rounded">
                        Review needed
                      </span>
                                        )}
                                    </div>
                                    <p className="text-sm font-medium text-gray-900 mb-1 truncate">
                                        {task.description}
                                    </p>
                                    <p className="text-xs text-gray-600 truncate">{task.repo_url}</p>
                                    <p className="text-xs text-gray-500 mt-1">
                                        Created: {new Date(task.created_at).toLocaleString()}
                                    </p>
                                </div>
                                {task.status !== 'completed' && task.status !== 'cancelled' && (
                                    <button
                                        onClick={(e) => handleCancel(task.id, e)}
                                        className="text-red-600 hover:text-red-800 text-sm font-medium"
                                    >
                                        Cancel
                                    </button>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
