import {useMemo} from 'react';
import {useSearchParams} from 'react-router-dom';
import type {TaskPriority, TaskStatus} from '../types';

const VALID_STATUSES: TaskStatus[] = [
    'created',
    'queued',
    'claimed',
    'in_progress',
    'awaiting_review',
    'pending_response',
    'completed',
    'cancelled',
    'failed',
];

const VALID_PRIORITIES: TaskPriority[] = ['low', 'normal', 'high', 'urgent'];

export interface QueueUrlState {
    filters: {
        status?: TaskStatus;
        priority?: TaskPriority;
        search?: string;
        worker_id?: string;
        project_id?: string;
    };
    page: number;
    selectedTaskId: string | null;
    setFilters: (filters: {
        status?: TaskStatus;
        priority?: TaskPriority;
        search?: string;
        worker_id?: string;
        project_id?: string;
    }) => void;
    setPage: (page: number) => void;
    setSelectedTaskId: (taskId: string | null) => void;
}

const readStatus = (value: string | null): TaskStatus | undefined => {
    if (!value) {
        return undefined;
    }

    return VALID_STATUSES.includes(value as TaskStatus) ? (value as TaskStatus) : undefined;
};

const readPriority = (value: string | null): TaskPriority | undefined => {
    if (!value) {
        return undefined;
    }

    return VALID_PRIORITIES.includes(value as TaskPriority) ? (value as TaskPriority) : undefined;
};

export const useQueueSearchParams = (): QueueUrlState => {
    const [searchParams, setSearchParams] = useSearchParams();

    const page = useMemo(() => {
        const rawPage = searchParams.get('page');
        if (!rawPage) {
            return 1;
        }

        const parsedPage = Number.parseInt(rawPage, 10);
        return Number.isFinite(parsedPage) && parsedPage > 0 ? parsedPage : 1;
    }, [searchParams]);

    const filters = useMemo(() => ({
        status: readStatus(searchParams.get('status')),
        priority: readPriority(searchParams.get('priority')),
        search: searchParams.get('search') || undefined,
        worker_id: searchParams.get('workerId') || undefined,
        project_id: searchParams.get('projectId') || undefined,
    }), [searchParams]);

    const selectedTaskId = searchParams.get('task');

    const setFilters = (nextFilters: QueueUrlState['filters']) => {
        const nextParams = new URLSearchParams(searchParams);
        const mappings: Array<[keyof QueueUrlState['filters'], string]> = [
            ['status', 'status'],
            ['priority', 'priority'],
            ['search', 'search'],
            ['worker_id', 'workerId'],
            ['project_id', 'projectId'],
        ];

        mappings.forEach(([filterKey, paramKey]) => {
            const value = nextFilters[filterKey];
            if (value) {
                nextParams.set(paramKey, value);
            } else {
                nextParams.delete(paramKey);
            }
        });

        nextParams.delete('page');
        nextParams.delete('task');
        setSearchParams(nextParams);
    };

    const setPage = (nextPage: number) => {
        const nextParams = new URLSearchParams(searchParams);
        if (nextPage <= 1) {
            nextParams.delete('page');
        } else {
            nextParams.set('page', String(nextPage));
        }
        setSearchParams(nextParams);
    };

    const setSelectedTaskId = (taskId: string | null) => {
        const nextParams = new URLSearchParams(searchParams);
        if (taskId) {
            nextParams.set('task', taskId);
        } else {
            nextParams.delete('task');
        }
        setSearchParams(nextParams);
    };

    return {
        filters,
        page,
        selectedTaskId,
        setFilters,
        setPage,
        setSelectedTaskId,
    };
};
