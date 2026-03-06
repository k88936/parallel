'use client';

import {useEffect, useState} from 'react';
import {useParams, useRouter} from 'next/navigation';
import type {Task, ReviewData, TaskStatus} from '@/types/task';
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

const STATUS_LABELS: Record<TaskStatus, string> = {
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

export default function TaskDetailPage() {
    const params = useParams();
    const router = useRouter();
    const taskId = params.id as string;

    const [task, setTask] = useState<Task | null>(null);
    const [reviewData, setReviewData] = useState<ReviewData | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [feedbackMessage, setFeedbackMessage] = useState('');
    const [submitting, setSubmitting] = useState(false);

    const fetchData = async () => {
        setError(null);
        try {
            const taskData = await api.getTask(taskId);
            setTask(taskData);

            const review = await api.getReviewData(taskId);
            setReviewData(review);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to fetch task');
        }
    };

    useEffect(() => {
        fetchData();
        const interval = setInterval(fetchData, 5000);
        return () => clearInterval(interval);
    }, []);

    const handleSubmitFeedback = async (feedbackType: 'approve' | 'request_changes' | 'abort') => {
        setSubmitting(true);
        try {
            await api.submitFeedback(taskId, {
                feedback_type: feedbackType,
                message: feedbackMessage || (feedbackType === 'approve' ? 'Approved' : ''),
            });
            setFeedbackMessage('');
            await fetchData();
        } catch (err) {
            alert(err instanceof Error ? err.message : 'Failed to submit feedback');
        } finally {
            setSubmitting(false);
        }
    };

    if (error) return <div className="min-h-screen bg-gray-100 p-8 text-red-600">Error: {error}</div>;
    if (!task) return <div className="min-h-screen bg-gray-100 p-8">Task not found</div>;

    return (
        <div className="min-h-screen bg-gray-100">
            <div className="container mx-auto px-4 py-8 max-w-4xl">
                <button
                    onClick={() => router.push('/')}
                    className="mb-4 text-blue-600 hover:text-blue-800 flex items-center gap-1"
                >
                    <span>&larr;</span> Back to tasks
                </button>

                <div className="bg-white rounded-lg shadow-md p-6 mb-6">
                    <div className="flex items-start justify-between mb-4">
                        <h1 className="text-2xl font-bold text-gray-900">Task Details</h1>
                        <span
                            className={`px-3 py-1 rounded text-sm font-medium ${
                                STATUS_COLORS[task.status]
                            }`}
                        >
              {STATUS_LABELS[task.status]}
            </span>
                    </div>

                    <div className="space-y-4">
                        <div>
                            <label className="block text-sm font-medium text-gray-600">Description</label>
                            <p className="text-gray-900">{task.description}</p>
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-600">Repository</label>
                            <p className="text-gray-900 font-mono text-sm">{task.repo_url}</p>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-600">Base Branch</label>
                                <p className="text-gray-900">{task.base_branch}</p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-600">Target Branch</label>
                                <p className="text-gray-900 font-mono text-sm">{task.target_branch}</p>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-600">Priority</label>
                                <p className="text-gray-900 capitalize">{task.priority}</p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-600">Created</label>
                                <p className="text-gray-900">{new Date(task.created_at).toLocaleString()}</p>
                            </div>
                        </div>

                        {task.claimed_by && (
                            <div>
                                <label className="block text-sm font-medium text-gray-600">Claimed By</label>
                                <p className="text-gray-900 font-mono text-sm">{task.claimed_by}</p>
                            </div>
                        )}
                    </div>
                </div>

                {reviewData && (
                    <div className="bg-white rounded-lg shadow-md p-6">
                        <h2 className="text-xl font-bold text-gray-900 mb-4">Review Required</h2>

                        {reviewData.messages.length > 0 && (
                            <div className="mb-6">
                                <h3 className="text-lg font-semibold text-gray-800 mb-3">Agent Messages</h3>
                                <div className="space-y-3 max-h-96 overflow-y-auto border rounded-lg p-4 bg-gray-50">
                                    {reviewData.messages.map((msg, idx) => (
                                        <div key={idx} className="border-b last:border-b-0 pb-3 last:pb-0">
                                            <div className="flex items-center gap-2 mb-1">
                        <span
                            className={`text-xs font-medium px-2 py-0.5 rounded ${
                                msg.role === 'assistant'
                                    ? 'bg-blue-100 text-blue-800'
                                    : msg.role === 'user'
                                        ? 'bg-green-100 text-green-800'
                                        : 'bg-gray-100 text-gray-800'
                            }`}
                        >
                          {msg.role}
                        </span>
                                                <span className="text-xs text-gray-500">
                          {new Date(msg.timestamp).toLocaleTimeString()}
                        </span>
                                            </div>
                                            <p className="text-sm text-gray-700 whitespace-pre-wrap">{msg.content}</p>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {reviewData.diff && (
                            <div className="mb-6">
                                <h3 className="text-lg font-semibold text-gray-800 mb-3">Changes (Diff)</h3>
                                <pre
                                    className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs font-mono max-h-96 overflow-y-auto">
                  {reviewData.diff}
                </pre>
                            </div>
                        )}

                        <div className="border-t pt-6">
                            <h3 className="text-lg font-semibold text-gray-800 mb-3">Provide Feedback</h3>

                            <div className="mb-4">
                                <label className="block text-sm font-medium text-gray-600 mb-2">
                                    Feedback Message (required for changes)
                                </label>
                                <textarea
                                    value={feedbackMessage}
                                    onChange={(e) => setFeedbackMessage(e.target.value)}
                                    className="w-full border rounded-lg px-3 py-2 text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    rows={3}
                                    placeholder="Describe what changes you'd like..."
                                />
                            </div>

                            <div className="flex flex-wrap gap-3">
                                <button
                                    onClick={() => handleSubmitFeedback('approve')}
                                    disabled={submitting}
                                    className="bg-green-600 hover:bg-green-700 text-white font-medium py-2 px-4 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {submitting ? 'Submitting...' : 'Approve & Push'}
                                </button>
                                <button
                                    onClick={() => handleSubmitFeedback('request_changes')}
                                    disabled={submitting || !feedbackMessage.trim()}
                                    className="bg-yellow-600 hover:bg-yellow-700 text-white font-medium py-2 px-4 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {submitting ? 'Submitting...' : 'Request Changes'}
                                </button>
                                <button
                                    onClick={() => handleSubmitFeedback('abort')}
                                    disabled={submitting}
                                    className="bg-red-600 hover:bg-red-700 text-white font-medium py-2 px-4 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {submitting ? 'Submitting...' : 'Abort Task'}
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {task.status === 'in_progress' && (
                    <div className="bg-white rounded-lg shadow-md p-6">
                        <div className="flex items-center gap-3">
                            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-orange-600"></div>
                            <p className="text-gray-600">Agent is working on this task...</p>
                        </div>
                    </div>
                )}

                {task.status === 'completed' && (
                    <div className="bg-green-50 border border-green-200 rounded-lg p-6">
                        <p className="text-green-800 font-medium">Task completed successfully!</p>
                    </div>
                )}

                {task.status === 'cancelled' && (
                    <div className="bg-red-50 border border-red-200 rounded-lg p-6">
                        <p className="text-red-800 font-medium">Task was cancelled.</p>
                    </div>
                )}
            </div>
        </div>
    );
}
