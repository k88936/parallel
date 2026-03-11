import { useEffect, useRef, useState, useCallback } from 'react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
    fetchTasks,
    fetchReviewData,
    cancelTask,
    retryTask,
    submitFeedback,
    toggleExpand,
    setFilters,
    setPage,
    collapseAll,
} from '../store/slices/tasksSlice';
import type { TaskStatus, TaskPriority, FeedbackType } from '../types';
import styles from './QueuePage.module.css';

import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Tag from '@jetbrains/ring-ui-built/components/tag/tag';
import Select from '@jetbrains/ring-ui-built/components/select/select';
import Input from '@jetbrains/ring-ui-built/components/input/input';
import Confirm from '@jetbrains/ring-ui-built/components/confirm/confirm';

const STATUS_OPTIONS = [
    { key: '', label: 'All Statuses' },
    { key: 'created', label: 'Created' },
    { key: 'queued', label: 'Queued' },
    { key: 'claimed', label: 'Claimed' },
    { key: 'in_progress', label: 'In Progress' },
    { key: 'awaiting_review', label: 'Awaiting Review' },
    { key: 'pending_response', label: 'Pending Response' },
    { key: 'completed', label: 'Completed' },
    { key: 'cancelled', label: 'Cancelled' },
    { key: 'failed', label: 'Failed' },
];

const PRIORITY_OPTIONS = [
    { key: '', label: 'All Priorities' },
    { key: 'low', label: 'Low' },
    { key: 'normal', label: 'Normal' },
    { key: 'high', label: 'High' },
    { key: 'urgent', label: 'Urgent' },
];

const getStatusColor = (status: TaskStatus): string => {
    switch (status) {
        case 'created':
            return styles.statusCreated;
        case 'queued':
            return styles.statusQueued;
        case 'claimed':
            return styles.statusClaimed;
        case 'in_progress':
            return styles.statusInProgress;
        case 'awaiting_review':
            return styles.statusAwaitingReview;
        case 'pending_response':
            return styles.statusPendingResponse;
        case 'completed':
            return styles.statusCompleted;
        case 'cancelled':
            return styles.statusCancelled;
        case 'failed':
            return styles.statusFailed;
        default:
            return '';
    }
};

const getPriorityColor = (priority: TaskPriority): string => {
    switch (priority) {
        case 'low':
            return styles.priorityLow;
        case 'normal':
            return styles.priorityNormal;
        case 'high':
            return styles.priorityHigh;
        case 'urgent':
            return styles.priorityUrgent;
        default:
            return '';
    }
};

const formatTimeAgo = (dateStr: string): string => {
    const date = new Date(dateStr);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h`;
    const days = Math.floor(hours / 24);
    return `${days}d`;
};

const shortenId = (id: string): string => {
    return id.substring(0, 8);
};

export const QueuePage = () => {
    const dispatch = useAppDispatch();
    const {
        tasks,
        total,
        hasMore,
        expandedTaskIds,
        filters,
        pagination,
        reviewData,
        loading,
        reviewLoading,
    } = useAppSelector((state) => state.tasks);

    const [searchInput, setSearchInput] = useState(filters.search || '');
    const [cancelConfirm, setCancelConfirm] = useState<string | null>(null);
    const [retryConfirm, setRetryConfirm] = useState<string | null>(null);

    const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

    const loadTasks = useCallback(() => {
        const query = {
            status: filters.status || null,
            priority: filters.priority || null,
            search: filters.search || null,
            worker_id: filters.worker_id || undefined,
            project_id: filters.project_id || undefined,
            limit: pagination.limit,
            offset: pagination.offset,
        };
        dispatch(fetchTasks(query));
    }, [dispatch, filters, pagination.limit, pagination.offset]);

    useEffect(() => {
        loadTasks();

        pollIntervalRef.current = setInterval(() => {
            loadTasks();
        }, 5000);

        return () => {
            if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current);
            }
        };
    }, [loadTasks]);

    useEffect(() => {
        expandedTaskIds.forEach((taskId) => {
            const task = tasks.find((t) => t.id === taskId);
            if (task?.status === 'awaiting_review' && !reviewData[taskId]) {
                dispatch(fetchReviewData(taskId));
            }
        });
    }, [expandedTaskIds, tasks, reviewData, dispatch]);

    const handleStatusChange = (option: { key: string } | null) => {
        dispatch(setFilters({ status: (option?.key as TaskStatus) || undefined }));
    };

    const handlePriorityChange = (option: { key: string } | null) => {
        dispatch(setFilters({ priority: (option?.key as TaskPriority) || undefined }));
    };

    const handleSearch = () => {
        dispatch(setFilters({ search: searchInput || undefined }));
    };

    const handleSearchKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            handleSearch();
        }
    };

    const handleRefresh = () => {
        loadTasks();
    };

    const handleExpand = (taskId: string) => {
        dispatch(toggleExpand(taskId));
    };

    const handleCancel = (taskId: string) => {
        setCancelConfirm(taskId);
    };

    const handleCancelConfirm = async () => {
        if (cancelConfirm) {
            await dispatch(cancelTask(cancelConfirm));
            setCancelConfirm(null);
        }
    };

    const handleRetry = (taskId: string) => {
        setRetryConfirm(taskId);
    };

    const handleRetryConfirm = async () => {
        if (retryConfirm) {
            await dispatch(retryTask({ id: retryConfirm, clearReviewData: false }));
            setRetryConfirm(null);
        }
    };

    const handleFeedback = async (taskId: string, feedbackType: FeedbackType, message: string) => {
        await dispatch(submitFeedback({ id: taskId, feedback: { feedback_type: feedbackType, message } }));
    };

    const handlePrevPage = () => {
        const newOffset = Math.max(0, pagination.offset - pagination.limit);
        dispatch(setPage(Math.floor(newOffset / pagination.limit)));
    };

    const handleNextPage = () => {
        if (hasMore) {
            dispatch(setPage(Math.floor((pagination.offset + pagination.limit) / pagination.limit)));
        }
    };

    const currentPage = Math.floor(pagination.offset / pagination.limit) + 1;
    const totalPages = Math.ceil(total / pagination.limit);

    const selectedStatusOption = STATUS_OPTIONS.find((o) => o.key === (filters.status || ''));
    const selectedPriorityOption = PRIORITY_OPTIONS.find((o) => o.key === (filters.priority || ''));

    return (
        <div className={styles.container}>
            <Island>
                <IslandHeader border>
                    <div className={styles.headerRow}>
                        <Heading level={1}>Task Queue</Heading>
                        <div className={styles.headerActions}>
                            <Button onClick={handleRefresh} disabled={loading}>
                                Refresh
                            </Button>
                        </div>
                    </div>
                </IslandHeader>
                <IslandContent>
                    <div className={styles.filterBar}>
                        <div className={styles.filterGroup}>
                            <Select
                                data={STATUS_OPTIONS}
                                selected={selectedStatusOption}
                                onSelect={handleStatusChange}
                                label="Status"
                                clear
                                type={Select.Type.INLINE}
                            />
                            <Select
                                data={PRIORITY_OPTIONS}
                                selected={selectedPriorityOption}
                                onSelect={handlePriorityChange}
                                label="Priority"
                                clear
                                type={Select.Type.INLINE}
                            />
                            <div className={styles.searchGroup}>
                                <Input
                                    value={searchInput}
                                    onChange={(e) => setSearchInput(e.target.value)}
                                    onKeyDown={handleSearchKeyDown}
                                    placeholder="Search tasks..."
                                />
                                <Button onClick={handleSearch}>Search</Button>
                            </div>
                        </div>
                        <Button onClick={() => dispatch(collapseAll())}>Collapse All</Button>
                    </div>

                    {loading && tasks.length === 0 ? (
                        <div className={styles.empty}>
                            <Loader />
                        </div>
                    ) : tasks.length === 0 ? (
                        <div className={styles.empty}>
                            <Text>No tasks found</Text>
                        </div>
                    ) : (
                        <div className={styles.tableContainer}>
                            <table className={styles.table}>
                                <thead>
                                    <tr>
                                        <th className={styles.colExpand}></th>
                                        <th className={styles.colId}>ID</th>
                                        <th className={styles.colTitle}>Title</th>
                                        <th className={styles.colStatus}>Status</th>
                                        <th className={styles.colPriority}>Priority</th>
                                        <th className={styles.colAge}>Age</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {tasks.map((task) => {
                                        const isExpanded = expandedTaskIds.includes(task.id);
                                        const taskReviewData = reviewData[task.id];

                                        return (
                                            <>
                                                <tr
                                                    key={task.id}
                                                    className={`${styles.row} ${isExpanded ? styles.rowExpanded : ''}`}
                                                    onClick={() => handleExpand(task.id)}
                                                >
                                                    <td className={styles.colExpand}>
                                                        <span className={styles.expandIcon}>
                                                            {isExpanded ? '▼' : '▶'}
                                                        </span>
                                                    </td>
                                                    <td className={styles.colId}>
                                                        <code>{shortenId(task.id)}</code>
                                                    </td>
                                                    <td className={styles.colTitle}>
                                                        <div className={styles.titleCell}>
                                                            {task.title}
                                                        </div>
                                                    </td>
                                                    <td className={styles.colStatus}>
                                                        <Tag className={getStatusColor(task.status)}>
                                                            {task.status.replace('_', ' ')}
                                                        </Tag>
                                                    </td>
                                                    <td className={styles.colPriority}>
                                                        <Tag className={getPriorityColor(task.priority)}>
                                                            {task.priority}
                                                        </Tag>
                                                    </td>
                                                    <td className={styles.colAge}>
                                                        {formatTimeAgo(task.created_at)}
                                                    </td>
                                                </tr>
                                                {isExpanded && (
                                                    <tr key={`${task.id}-expanded`} className={styles.expandedRow}>
                                                        <td colSpan={6}>
                                                            <div className={styles.expandedContent}>
                                                                <div className={styles.detailGrid}>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Description</span>
                                                                        <span className={styles.detailValue}>
                                                                            {task.description || 'No description'}
                                                                        </span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Repository</span>
                                                                        <span className={styles.detailValue}>
                                                                            <code>{task.repo_url}</code>
                                                                        </span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Base Branch</span>
                                                                        <span className={styles.detailValue}>
                                                                            <code>{task.base_branch}</code>
                                                                        </span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Target Branch</span>
                                                                        <span className={styles.detailValue}>
                                                                            <code>{task.target_branch}</code>
                                                                        </span>
                                                                    </div>
                                                                    {task.claimed_by && (
                                                                        <div className={styles.detailItem}>
                                                                            <span className={styles.detailLabel}>Worker</span>
                                                                            <span className={styles.detailValue}>
                                                                                <code>{shortenId(task.claimed_by)}</code>
                                                                            </span>
                                                                        </div>
                                                                    )}
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Created</span>
                                                                        <span className={styles.detailValue}>
                                                                            {new Date(task.created_at).toLocaleString()}
                                                                        </span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Updated</span>
                                                                        <span className={styles.detailValue}>
                                                                            {new Date(task.updated_at).toLocaleString()}
                                                                        </span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Max Execution</span>
                                                                        <span className={styles.detailValue}>
                                                                            {task.max_execution_time}s
                                                                        </span>
                                                                    </div>
                                                                </div>

                                                                {Object.keys(task.required_labels).length > 0 && (
                                                                    <div className={styles.labelsSection}>
                                                                        <span className={styles.detailLabel}>Required Labels</span>
                                                                        <div className={styles.labelTags}>
                                                                            {Object.entries(task.required_labels).map(([key, value]) => (
                                                                                <Tag key={key}>{key}: {value}</Tag>
                                                                            ))}
                                                                        </div>
                                                                    </div>
                                                                )}

                                                                <div className={styles.actionsRow}>
                                                                    {(task.status === 'queued' || task.status === 'created' || task.status === 'in_progress' || task.status === 'claimed') && (
                                                                        <Button danger onClick={(e) => { e.stopPropagation(); handleCancel(task.id); }}>
                                                                            Cancel
                                                                        </Button>
                                                                    )}
                                                                    {(task.status === 'failed' || task.status === 'cancelled') && (
                                                                        <Button onClick={(e) => { e.stopPropagation(); handleRetry(task.id); }}>
                                                                            Retry
                                                                        </Button>
                                                                    )}
                                                                </div>

                                                                {task.status === 'awaiting_review' && (
                                                                    <div className={styles.reviewSection}>
                                                                        <Heading level={4}>Review Required</Heading>
                                                                        {reviewLoading && !taskReviewData ? (
                                                                            <Loader />
                                                                        ) : taskReviewData ? (
                                                                            <>
                                                                                {taskReviewData.messages.length > 0 && (
                                                                                    <div className={styles.messagesSection}>
                                                                                        <Heading level={4}>Messages</Heading>
                                                                                        <div className={styles.messagesList}>
                                                                                            {taskReviewData.messages.map((msg: { role: string; timestamp: string; content: string }, idx: number) => (
                                                                                                <div key={idx} className={styles.messageItem}>
                                                                                                    <div className={styles.messageHeader}>
                                                                                                        <Tag>{msg.role}</Tag>
                                                                                                        <span className={styles.messageTime}>
                                                                                                            {new Date(msg.timestamp).toLocaleTimeString()}
                                                                                                        </span>
                                                                                                    </div>
                                                                                                    <div className={styles.messageContent}>
                                                                                                        {msg.content}
                                                                                                    </div>
                                                                                                </div>
                                                                                            ))}
                                                                                        </div>
                                                                                    </div>
                                                                                )}
                                                                                {taskReviewData.diff && (
                                                                                    <div className={styles.diffSection}>
                                                                                        <Heading level={4}>Diff</Heading>
                                                                                        <pre className={styles.diffContent}>
                                                                                            {taskReviewData.diff}
                                                                                        </pre>
                                                                                    </div>
                                                                                )}
                                                                                <div className={styles.feedbackForm}>
                                                                                    <Heading level={4}>Provide Feedback</Heading>
                                                                                    <div className={styles.feedbackButtons}>
                                                                                        <Button
                                                                                            primary
                                                                                            onClick={() => handleFeedback(task.id, 'approve', 'Approved')}
                                                                                        >
                                                                                            Approve
                                                                                        </Button>
                                                                                        <Button
                                                                                            onClick={() => handleFeedback(task.id, 'request_changes', 'Changes requested')}
                                                                                        >
                                                                                            Request Changes
                                                                                        </Button>
                                                                                        <Button
                                                                                            danger
                                                                                            onClick={() => handleFeedback(task.id, 'abort', 'Task aborted')}
                                                                                        >
                                                                                            Abort
                                                                                        </Button>
                                                                                    </div>
                                                                                </div>
                                                                            </>
                                                                        ) : (
                                                                            <Text>No review data available</Text>
                                                                        )}
                                                                    </div>
                                                                )}
                                                            </div>
                                                        </td>
                                                    </tr>
                                                )}
                                            </>
                                        );
                                    })}
                                </tbody>
                            </table>
                        </div>
                    )}

                    {total > 0 && (
                        <div className={styles.pagination}>
                            <Button disabled={pagination.offset === 0} onClick={handlePrevPage}>
                                Previous
                            </Button>
                            <Text>
                                Page {currentPage} of {totalPages} ({total} total)
                            </Text>
                            <Button disabled={!hasMore} onClick={handleNextPage}>
                                Next
                            </Button>
                        </div>
                    )}
                </IslandContent>
            </Island>

            <Confirm
                show={cancelConfirm !== null}
                text="Are you sure you want to cancel this task?"
                confirmLabel="Cancel Task"
                rejectLabel="Keep Running"
                onConfirm={handleCancelConfirm}
                onReject={() => setCancelConfirm(null)}
            />

            <Confirm
                show={retryConfirm !== null}
                text="Are you sure you want to retry this task?"
                confirmLabel="Retry Task"
                rejectLabel="Cancel"
                onConfirm={handleRetryConfirm}
                onReject={() => setRetryConfirm(null)}
            />
        </div>
    );
};
