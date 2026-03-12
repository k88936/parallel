import {useEffect, useMemo, useRef, useState} from 'react';
import {useQueueSearchParams} from '../hooks/useQueueSearchParams';
import {useTasksStore} from '../stores/useTasksStore';
import type {FeedbackType, ListTasksQuery, TaskPriority, TaskStatus} from '../types';

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
import ButtonGroup from '@jetbrains/ring-ui-built/components/button-group/button-group';
import Group from '@jetbrains/ring-ui-built/components/group/group';
import Pager from '@jetbrains/ring-ui-built/components/pager/pager';
import Selection from '@jetbrains/ring-ui-built/components/table/selection';
import TableContainer from '@jetbrains/ring-ui-built/components/table/table';
import type {Column, SortParams} from '@jetbrains/ring-ui-built/components/table/header-cell';

const PAGE_SIZE = 20;
const PRIORITY_ORDER: Record<TaskPriority, number> = {
    low: 0,
    normal: 1,
    high: 2,
    urgent: 3,
};

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
        case 'created': return '!bg-[#616161]';
        case 'queued': return '!bg-[#2196f3]';
        case 'claimed': return '!bg-[#00bcd4]';
        case 'in_progress': return '!bg-[#ff9800]';
        case 'awaiting_review': return '!bg-[#ff5722]';
        case 'pending_response': return '!bg-[#9c27b0]';
        case 'completed': return '!bg-[#4caf50]';
        case 'cancelled': return '!bg-transparent !border !border-[#f44336] !text-[#f44336]';
        case 'failed': return '!bg-[#f44336]';
        default: return '';
    }
};

const getPriorityColor = (priority: TaskPriority): string => {
    switch (priority) {
        case 'low': return '!bg-[#616161]';
        case 'normal': return '!bg-[#2196f3]';
        case 'high': return '!bg-[#ff9800]';
        case 'urgent': return '!bg-[#f44336]';
        default: return '';
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
    const [sortKey, setSortKey] = useState('created_at');
    const [sortOrder, setSortOrder] = useState(false);
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
    const selectedStatusOption = STATUS_OPTIONS.find((option) => option.key === (filters.status || ''));
    const selectedPriorityOption = PRIORITY_OPTIONS.find((option) => option.key === (filters.priority || ''));
    type QueueTask = (typeof tasks)[number];

    const sortedTasks = useMemo(() => {
        const nextTasks = [...tasks];
        const direction = sortOrder ? 1 : -1;

        nextTasks.sort((left, right) => {
            let comparison = 0;

            switch (sortKey) {
                case 'id':
                    comparison = left.id.localeCompare(right.id);
                    break;
                case 'title':
                    comparison = left.title.localeCompare(right.title);
                    break;
                case 'status':
                    comparison = left.status.localeCompare(right.status);
                    break;
                case 'priority':
                    comparison = PRIORITY_ORDER[left.priority] - PRIORITY_ORDER[right.priority];
                    break;
                case 'created_at':
                    comparison = new Date(left.created_at).getTime() - new Date(right.created_at).getTime();
                    break;
                default:
                    comparison = 0;
                    break;
            }

            if (comparison === 0) {
                comparison = left.id.localeCompare(right.id);
            }

            return comparison * direction;
        });

        return nextTasks;
    }, [sortKey, sortOrder, tasks]);

    const selectedTask = selectedTaskId ? tasks.find((task) => task.id === selectedTaskId) ?? null : null;
    const selectedTaskReviewData = selectedTask ? reviewData[selectedTask.id] : null;
    const selectedTaskReviewLoading = selectedTask ? reviewLoadingIds[selectedTask.id] ?? false : false;

    const handleSort = ({column, order}: SortParams) => {
        setSortKey(column.id);
        setSortOrder(order);
    };

    const columns = useMemo<Column<QueueTask>[]>(() => [
        {
            id: 'id',
            sortable: true,
            title: 'ID',
            className: 'align-top whitespace-nowrap',
            getValue: (task) => (
                <code className="font-mono text-xs text-[var(--ring-text-color,#fff)]">
                    {shortenId(task.id)}
                </code>
            ),
        },
        {
            id: 'title',
            sortable: true,
            title: 'Title',
            className: 'align-top',
            getValue: (task) => (
                <div className="max-w-[400px] overflow-hidden text-ellipsis whitespace-nowrap">
                    {task.title}
                </div>
            ),
        },
        {
            id: 'status',
            sortable: true,
            title: 'Status',
            className: 'align-top whitespace-nowrap',
            getValue: (task) => (
                <Tag className={getStatusColor(task.status)}>
                    {task.status.replace('_', ' ')}
                </Tag>
            ),
        },
        {
            id: 'priority',
            sortable: true,
            title: 'Priority',
            className: 'align-top whitespace-nowrap',
            getValue: (task) => (
                <Tag className={getPriorityColor(task.priority)}>
                    {task.priority}
                </Tag>
            ),
        },
        {
            id: 'created_at',
            sortable: true,
            title: 'Age',
            className: 'align-top whitespace-nowrap text-[var(--ring-secondary-text-color,#888)]',
            getValue: (task) => formatTimeAgo(task.created_at),
        },
    ], []);

    const tableSelection = useMemo(() => new Selection<QueueTask>({
        data: sortedTasks,
        focused: selectedTask,
        getKey: (task) => task.id,
    }), [selectedTask, sortedTasks]);

    return (
        <Group className="p-4 overflow-auto flex-1">
            <Island>
                <IslandHeader border>
                    <Heading level={1}>Task Queue</Heading>
                </IslandHeader>
                <IslandContent>
                    <Group className="flex justify-between m-2">
                        <Group className="flex">
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
                        </Group>

                        <ButtonGroup>
                            <Button onClick={() => setSelectedTaskId(null)}>Collapse All</Button>
                            <Button onClick={() => void refreshTasks()} disabled={loading}>
                                Refresh
                            </Button>
                        </ButtonGroup>
                    </Group>

                    {error && tasks.length === 0 ? (
                        <div className="flex items-center justify-center h-50 text-(--ring-secondary-text-color)">
                            <Text>{error}</Text>
                        </div>
                    ) : loading && tasks.length === 0 ? (
                        <div className="flex items-center justify-center h-50 text-(--ring-secondary-text-color)">
                            <Loader />
                        </div>
                    ) : tasks.length === 0 ? (
                        <div className="flex items-center justify-center h-50 text-(--ring-secondary-text-color)">
                            <Text>No tasks found</Text>
                        </div>
                    ) : (
                        <>
                            <TableContainer<QueueTask>
                                className="mb-4"
                                columns={columns}
                                data={sortedTasks}
                                getItemClassName={(task) => [
                                    'cursor-pointer transition-colors',
                                    task.id === selectedTaskId
                                        ? 'bg-[var(--ring-selected-background-color,#2a2a2a)]'
                                        : 'hover:bg-[var(--ring-hover-background-color,#2d2d2d)]',
                                ].join(' ')}
                                getItemKey={(task) => task.id}
                                onItemClick={(task) => handleExpand(task.id)}
                                onSort={handleSort}
                                renderEmpty={() => <Text>No tasks found</Text>}
                                selectable={false}
                                selection={tableSelection}
                                sortKey={sortKey}
                                sortOrder={sortOrder}
                                stickyHeader={false}
                            />

                            {selectedTask && (
                                <div className="mb-4 rounded border border-[var(--ring-border-color,#3d3d3d)] bg-[var(--ring-selected-background-color,#252525)] p-4 pb-6">
                                    <div className="mb-4 flex items-center justify-between gap-4">
                                        <div>
                                            <Heading level={3}>{selectedTask.title}</Heading>
                                            <Text className="text-[var(--ring-secondary-text-color,#888)]">
                                                Task {shortenId(selectedTask.id)}
                                            </Text>
                                        </div>
                                        <Button onClick={() => setSelectedTaskId(null)}>Close Details</Button>
                                    </div>

                                    <div className="grid grid-cols-3 gap-4 mb-4">
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Description</span>
                                            <span className="text-sm">{selectedTask.description || 'No description'}</span>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Repository</span>
                                            <span className="text-sm">
                                                <code className="font-mono text-xs bg-[var(--ring-sidebar-background-color,#1e1e1e)] px-1.5 py-0.5 rounded-[3px] break-all">{selectedTask.repo_url}</code>
                                            </span>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Base Branch</span>
                                            <span className="text-sm">
                                                <code className="font-mono text-xs bg-[var(--ring-sidebar-background-color,#1e1e1e)] px-1.5 py-0.5 rounded-[3px] break-all">{selectedTask.base_branch}</code>
                                            </span>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Target Branch</span>
                                            <span className="text-sm">
                                                <code className="font-mono text-xs bg-[var(--ring-sidebar-background-color,#1e1e1e)] px-1.5 py-0.5 rounded-[3px] break-all">{selectedTask.target_branch}</code>
                                            </span>
                                        </div>
                                        {selectedTask.claimed_by && (
                                            <div className="flex flex-col gap-1">
                                                <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Worker</span>
                                                <span className="text-sm">
                                                    <code className="font-mono text-xs bg-[var(--ring-sidebar-background-color,#1e1e1e)] px-1.5 py-0.5 rounded-[3px] break-all">{shortenId(selectedTask.claimed_by)}</code>
                                                </span>
                                            </div>
                                        )}
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Created</span>
                                            <span className="text-sm">{new Date(selectedTask.created_at).toLocaleString()}</span>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Updated</span>
                                            <span className="text-sm">{new Date(selectedTask.updated_at).toLocaleString()}</span>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Max Execution</span>
                                            <span className="text-sm">{selectedTask.max_execution_time}s</span>
                                        </div>
                                    </div>

                                    {Object.keys(selectedTask.required_labels).length > 0 && (
                                        <div className="mb-4">
                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">Required Labels</span>
                                            <div className="flex flex-wrap gap-1.5 mt-2">
                                                {Object.entries(selectedTask.required_labels).map(([labelKey, labelValue]) => (
                                                    <Tag key={labelKey}>{`${labelKey}: ${labelValue}`}</Tag>
                                                ))}
                                            </div>
                                        </div>
                                    )}

                                    <div className="flex gap-2 mb-4">
                                        {(selectedTask.status === 'queued' ||
                                            selectedTask.status === 'created' ||
                                            selectedTask.status === 'in_progress' ||
                                            selectedTask.status === 'claimed') && (
                                            <Button danger onClick={() => setCancelConfirm(selectedTask.id)}>
                                                Cancel
                                            </Button>
                                        )}
                                        {(selectedTask.status === 'failed' || selectedTask.status === 'cancelled') && (
                                            <Button onClick={() => setRetryConfirm(selectedTask.id)}>
                                                Retry
                                            </Button>
                                        )}
                                    </div>

                                    {selectedTask.status === 'awaiting_review' && (
                                        <div className="mt-4 pt-4 border-t border-[var(--ring-border-color,#3d3d3d)]">
                                            <Heading level={4}>Review Required</Heading>
                                            {selectedTaskReviewLoading && !selectedTaskReviewData ? (
                                                <Loader />
                                            ) : selectedTaskReviewData ? (
                                                <>
                                                    {selectedTaskReviewData.messages.length > 0 && (
                                                        <div className="mb-4">
                                                            <Heading level={4}>Messages</Heading>
                                                            <div className="max-h-[300px] overflow-y-auto mt-2">
                                                                {selectedTaskReviewData.messages.map((message, index) => (
                                                                    <div key={index} className="p-3 bg-[var(--ring-sidebar-background-color,#1e1e1e)] rounded mb-2 last:mb-0">
                                                                        <div className="flex items-center gap-2 mb-2">
                                                                            <Tag>{message.role}</Tag>
                                                                            <span className="text-xs text-[var(--ring-secondary-text-color,#888)]">
                                                                                {new Date(message.timestamp).toLocaleTimeString()}
                                                                            </span>
                                                                        </div>
                                                                        <div className="text-sm whitespace-pre-wrap break-words">{message.content}</div>
                                                                    </div>
                                                                ))}
                                                            </div>
                                                        </div>
                                                    )}
                                                    {selectedTaskReviewData.diff && (
                                                        <div className="mb-4">
                                                            <Heading level={4}>Diff</Heading>
                                                            <pre className="max-h-[300px] overflow-auto bg-[var(--ring-sidebar-background-color,#1e1e1e)] p-3 rounded font-mono text-xs mt-2 whitespace-pre-wrap break-words">{selectedTaskReviewData.diff}</pre>
                                                        </div>
                                                    )}
                                                    <div className="mt-4 pt-4 border-t border-[var(--ring-border-color,#3d3d3d)]">
                                                        <Heading level={4}>Provide Feedback</Heading>
                                                        <div className="flex gap-2 mt-2">
                                                            <Button primary onClick={() => void handleFeedback(selectedTask.id, 'approve', 'Approved')}>
                                                                Approve
                                                            </Button>
                                                            <Button onClick={() => void handleFeedback(selectedTask.id, 'request_changes', 'Changes requested')}>
                                                                Request Changes
                                                            </Button>
                                                            <Button danger onClick={() => void handleFeedback(selectedTask.id, 'abort', 'Task aborted')}>
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
                            )}
                        </>
                    )}

                    {total > 0 && (
                        <div className="pt-4">
                            <Pager
                                currentPage={currentPage}
                                disablePageSizeSelector
                                onPageChange={(nextPage) => setPage(nextPage)}
                                pageSize={PAGE_SIZE}
                                total={total}
                            />
                            <Text className="mt-2 text-[var(--ring-secondary-text-color,#888)]">
                                Showing {tasks.length} tasks on page {currentPage}
                            </Text>
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
        </Group>
    );
};
