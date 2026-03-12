import {Fragment, useEffect, useMemo, useRef, useState} from 'react';
import {useQueueSearchParams} from '../hooks/useQueueSearchParams';
import {useTasksStore} from '../stores/useTasksStore';
import type {FeedbackType, ListTasksQuery, TaskPriority, TaskStatus} from '../types';
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

const PAGE_SIZE = 20;

const STATUS_OPTIONS = [
    {key: '', label: 'All Statuses'},
    {key: 'created', label: 'Created'},
    {key: 'queued', label: 'Queued'},
    {key: 'claimed', label: 'Claimed'},
    {key: 'in_progress', label: 'In Progress'},
    {key: 'awaiting_review', label: 'Awaiting Review'},
    {key: 'pending_response', label: 'Pending Response'},
    {key: 'completed', label: 'Completed'},
    {key: 'cancelled', label: 'Cancelled'},
    {key: 'failed', label: 'Failed'},
];

const PRIORITY_OPTIONS = [
    {key: '', label: 'All Priorities'},
    {key: 'low', label: 'Low'},
    {key: 'normal', label: 'Normal'},
    {key: 'high', label: 'High'},
    {key: 'urgent', label: 'Urgent'},
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

const shortenId = (id: string): string => id.substring(0, 8);

export const QueuePage = () => {
    const {filters, page, selectedTaskId, setFilters, setPage, setSelectedTaskId} = useQueueSearchParams();
    const tasks = useTasksStore((state) => state.tasks);
    const total = useTasksStore((state) => state.total);
    const hasMore = useTasksStore((state) => state.hasMore);
    const reviewData = useTasksStore((state) => state.reviewData);
    const reviewLoadingIds = useTasksStore((state) => state.reviewLoadingIds);
    const loading = useTasksStore((state) => state.loading);
    const error = useTasksStore((state) => state.error);
    const fetchTasks = useTasksStore((state) => state.fetchTasks);
    const refreshTasks = useTasksStore((state) => state.refreshTasks);
    const fetchReviewData = useTasksStore((state) => state.fetchReviewData);
    const cancelTask = useTasksStore((state) => state.cancelTask);
    const retryTask = useTasksStore((state) => state.retryTask);
    const submitFeedback = useTasksStore((state) => state.submitFeedback);

    const [cancelConfirm, setCancelConfirm] = useState<string | null>(null);
    const [retryConfirm, setRetryConfirm] = useState<string | null>(null);
    const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
    const currentSearch = filters.search || '';
    const searchInputRef = useRef(currentSearch);
    const appliedSearchRef = useRef(currentSearch);

    const query = useMemo<ListTasksQuery>(() => ({
        status: filters.status ?? null,
        priority: filters.priority ?? null,
        repo_url: null,
        worker_id: filters.worker_id,
        search: filters.search ?? null,
        sort_by: null,
        sort_direction: null,
        cursor: null,
        limit: PAGE_SIZE,
        offset: (page - 1) * PAGE_SIZE,
        project_id: filters.project_id,
    }), [filters, page]);

    useEffect(() => {
        appliedSearchRef.current = currentSearch;
        searchInputRef.current = currentSearch;
    }, [currentSearch]);

    useEffect(() => {
        void fetchTasks(query);
    }, [fetchTasks, query]);

    useEffect(() => {
        pollIntervalRef.current = setInterval(() => {
            void refreshTasks();
        }, 5000);

        return () => {
            if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current);
            }
        };
    }, [refreshTasks]);

    useEffect(() => {
        if (!selectedTaskId) {
            return;
        }

        const selectedTask = tasks.find((task) => task.id === selectedTaskId);
        if (selectedTask?.status === 'awaiting_review' && !reviewData[selectedTaskId]) {
            void fetchReviewData(selectedTaskId);
        }
    }, [fetchReviewData, reviewData, selectedTaskId, tasks]);

    const handleStatusChange = (option: {key: string} | null) => {
        setFilters({
            ...filters,
            status: option?.key ? option.key as TaskStatus : undefined,
        });
    };

    const handlePriorityChange = (option: {key: string} | null) => {
        setFilters({
            ...filters,
            priority: option?.key ? option.key as TaskPriority : undefined,
        });
    };

    const handleSearch = () => {
        setFilters({
            ...filters,
            search: searchInputRef.current || undefined,
        });
    };

    const handleSearchKeyDown = (event: React.KeyboardEvent) => {
        if (event.key === 'Enter') {
            handleSearch();
        }
    };

    const handleExpand = (taskId: string) => {
        setSelectedTaskId(selectedTaskId === taskId ? null : taskId);
    };

    const handleCancelConfirm = async () => {
        if (cancelConfirm) {
            await cancelTask(cancelConfirm);
            if (selectedTaskId === cancelConfirm) {
                setSelectedTaskId(null);
            }
            setCancelConfirm(null);
        }
    };

    const handleRetryConfirm = async () => {
        if (retryConfirm) {
            await retryTask(retryConfirm, false);
            setRetryConfirm(null);
        }
    };

    const handleFeedback = async (taskId: string, feedbackType: FeedbackType, message: string) => {
        await submitFeedback(taskId, {feedback_type: feedbackType, message});
    };

    const currentPage = page;
    const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));
    const selectedStatusOption = STATUS_OPTIONS.find((option) => option.key === (filters.status || ''));
    const selectedPriorityOption = PRIORITY_OPTIONS.find((option) => option.key === (filters.priority || ''));

    return (
        <div className={styles.container}>
            <Island>
                <IslandHeader border>
                    <div className={styles.headerRow}>
                        <Heading level={1}>Task Queue</Heading>
                        <div className={styles.headerActions}>
                            <Button onClick={() => void refreshTasks()} disabled={loading}>
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
                                    key={currentSearch}
                                    defaultValue={currentSearch}
                                    onChange={(event) => {
                                        searchInputRef.current = event.target.value;
                                    }}
                                    onKeyDown={handleSearchKeyDown}
                                    placeholder="Search tasks..."
                                />
                                <Button onClick={handleSearch}>Search</Button>
                            </div>
                        </div>
                        <Button onClick={() => setSelectedTaskId(null)}>Collapse All</Button>
                    </div>

                    {error && tasks.length === 0 ? (
                        <div className={styles.empty}>
                            <Text>{error}</Text>
                        </div>
                    ) : loading && tasks.length === 0 ? (
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
                                        const isExpanded = selectedTaskId === task.id;
                                        const taskReviewData = reviewData[task.id];
                                        const reviewLoading = reviewLoadingIds[task.id] ?? false;

                                        return (
                                            <Fragment key={task.id}>
                                                <tr
                                                    className={`${styles.row} ${isExpanded ? styles.rowExpanded : ''}`}
                                                    onClick={() => handleExpand(task.id)}
                                                >
                                                    <td className={styles.colExpand}>
                                                        <span className={styles.expandIcon}>{isExpanded ? '▼' : '▶'}</span>
                                                    </td>
                                                    <td className={styles.colId}>
                                                        <code>{shortenId(task.id)}</code>
                                                    </td>
                                                    <td className={styles.colTitle}>
                                                        <div className={styles.titleCell}>{task.title}</div>
                                                    </td>
                                                    <td className={styles.colStatus}>
                                                        <Tag className={getStatusColor(task.status)}>{task.status.replace('_', ' ')}</Tag>
                                                    </td>
                                                    <td className={styles.colPriority}>
                                                        <Tag className={getPriorityColor(task.priority)}>{task.priority}</Tag>
                                                    </td>
                                                    <td className={styles.colAge}>{formatTimeAgo(task.created_at)}</td>
                                                </tr>
                                                {isExpanded && (
                                                    <tr className={styles.expandedRow}>
                                                        <td colSpan={6}>
                                                            <div className={styles.expandedContent}>
                                                                <div className={styles.detailGrid}>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Description</span>
                                                                        <span className={styles.detailValue}>{task.description || 'No description'}</span>
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
                                                                        <span className={styles.detailValue}>{new Date(task.created_at).toLocaleString()}</span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Updated</span>
                                                                        <span className={styles.detailValue}>{new Date(task.updated_at).toLocaleString()}</span>
                                                                    </div>
                                                                    <div className={styles.detailItem}>
                                                                        <span className={styles.detailLabel}>Max Execution</span>
                                                                        <span className={styles.detailValue}>{task.max_execution_time}s</span>
                                                                    </div>
                                                                </div>

                                                                {Object.keys(task.required_labels).length > 0 && (
                                                                    <div className={styles.labelsSection}>
                                                                        <span className={styles.detailLabel}>Required Labels</span>
                                                                        <div className={styles.labelTags}>
                                                                            {Object.entries(task.required_labels).map(([labelKey, labelValue]) => (
                                                                                <Tag key={labelKey}>{`${labelKey}: ${labelValue}`}</Tag>
                                                                            ))}
                                                                        </div>
                                                                    </div>
                                                                )}

                                                                <div className={styles.actionsRow}>
                                                                    {(task.status === 'queued' ||
                                                                        task.status === 'created' ||
                                                                        task.status === 'in_progress' ||
                                                                        task.status === 'claimed') && (
                                                                        <Button
                                                                            danger
                                                                            onClick={(event) => {
                                                                                event.stopPropagation();
                                                                                setCancelConfirm(task.id);
                                                                            }}
                                                                        >
                                                                            Cancel
                                                                        </Button>
                                                                    )}
                                                                    {(task.status === 'failed' || task.status === 'cancelled') && (
                                                                        <Button
                                                                            onClick={(event) => {
                                                                                event.stopPropagation();
                                                                                setRetryConfirm(task.id);
                                                                            }}
                                                                        >
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
                                                                                            {taskReviewData.messages.map((message, index) => (
                                                                                                <div key={index} className={styles.messageItem}>
                                                                                                    <div className={styles.messageHeader}>
                                                                                                        <Tag>{message.role}</Tag>
                                                                                                        <span className={styles.messageTime}>
                                                                                                            {new Date(message.timestamp).toLocaleTimeString()}
                                                                                                        </span>
                                                                                                    </div>
                                                                                                    <div className={styles.messageContent}>{message.content}</div>
                                                                                                </div>
                                                                                            ))}
                                                                                        </div>
                                                                                    </div>
                                                                                )}
                                                                                {taskReviewData.diff && (
                                                                                    <div className={styles.diffSection}>
                                                                                        <Heading level={4}>Diff</Heading>
                                                                                        <pre className={styles.diffContent}>{taskReviewData.diff}</pre>
                                                                                    </div>
                                                                                )}
                                                                                <div className={styles.feedbackForm}>
                                                                                    <Heading level={4}>Provide Feedback</Heading>
                                                                                    <div className={styles.feedbackButtons}>
                                                                                        <Button primary onClick={() => void handleFeedback(task.id, 'approve', 'Approved')}>
                                                                                            Approve
                                                                                        </Button>
                                                                                        <Button onClick={() => void handleFeedback(task.id, 'request_changes', 'Changes requested')}>
                                                                                            Request Changes
                                                                                        </Button>
                                                                                        <Button danger onClick={() => void handleFeedback(task.id, 'abort', 'Task aborted')}>
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
                                            </Fragment>
                                        );
                                    })}
                                </tbody>
                            </table>
                        </div>
                    )}

                    {total > 0 && (
                        <div className={styles.pagination}>
                            <Button disabled={currentPage === 1} onClick={() => setPage(currentPage - 1)}>
                                Previous
                            </Button>
                            <Text>
                                Page {currentPage} of {totalPages} ({total} total)
                            </Text>
                            <Button disabled={!hasMore} onClick={() => setPage(currentPage + 1)}>
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
                onConfirm={() => void handleCancelConfirm()}
                onReject={() => setCancelConfirm(null)}
            />

            <Confirm
                show={retryConfirm !== null}
                text="Are you sure you want to retry this task?"
                confirmLabel="Retry Task"
                rejectLabel="Cancel"
                onConfirm={() => void handleRetryConfirm()}
                onReject={() => setRetryConfirm(null)}
            />
        </div>
    );
};
