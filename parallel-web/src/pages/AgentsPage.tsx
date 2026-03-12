import {useCallback, useEffect, useRef, useState} from 'react';
import {workersApi} from '../api';
import type {ResourceMonitor, WorkerInfo, WorkerStatus, WorkerSummary} from '../types';

import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Tag from '@jetbrains/ring-ui-built/components/tag/tag';

const STATUS_DOT_COLOR: Record<WorkerStatus, string> = {
    idle: 'bg-[#4caf50]',
    busy: 'bg-[#ff9800]',
    offline: 'bg-[#9e9e9e]',
    dead: 'bg-[#f44336]',
};

const formatTimeAgo = (dateStr: string): string => {
    const date = new Date(dateStr);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
};

const getResourceBarColor = (percent: number): string => {
    if (percent < 50) return 'bg-[#4caf50]';
    if (percent < 80) return 'bg-[#ff9800]';
    return 'bg-[#f44336]';
};

const getErrorMessage = (error: unknown): string => {
    if (error instanceof Error) {
        return error.message;
    }

    return 'Request failed';
};

export const AgentsPage = () => {
    const [workers, setWorkers] = useState<WorkerSummary[]>([]);
    const [selectedWorkerId, setSelectedWorkerId] = useState<string | null>(null);
    const [selectedWorkerInfo, setSelectedWorkerInfo] = useState<WorkerInfo | null>(null);
    const [selectedWorkerResources, setSelectedWorkerResources] = useState<ResourceMonitor | null>(null);
    const [loading, setLoading] = useState(false);
    const [infoLoading, setInfoLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

    const loadWorkers = useCallback(async () => {
        setLoading(true);
        try {
            const nextWorkers = await workersApi.list();
            setWorkers(nextWorkers);
            setError(null);
            setSelectedWorkerId((current) => {
                if (!current) {
                    return nextWorkers[0]?.id ?? null;
                }

                return nextWorkers.some((worker) => worker.id === current) ? current : nextWorkers[0]?.id ?? null;
            });
        } catch (error) {
            setError(getErrorMessage(error));
        } finally {
            setLoading(false);
        }
    }, []);

    const loadSelectedWorkerDetails = useCallback(async (workerId: string) => {
        setInfoLoading(true);
        try {
            const [workerInfo, workerResources] = await Promise.all([
                workersApi.getInfo(workerId),
                workersApi.getResources(workerId),
            ]);
            setSelectedWorkerInfo(workerInfo);
            setSelectedWorkerResources(workerResources);
            setError(null);
        } catch (error) {
            setError(getErrorMessage(error));
        } finally {
            setInfoLoading(false);
        }
    }, []);

    useEffect(() => {
        void loadWorkers();

        pollIntervalRef.current = setInterval(() => {
            void loadWorkers();
        }, 5000);

        return () => {
            if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current);
            }
        };
    }, [loadWorkers]);

    useEffect(() => {
        if (!selectedWorkerId) {
            setSelectedWorkerInfo(null);
            setSelectedWorkerResources(null);
            return;
        }

        void loadSelectedWorkerDetails(selectedWorkerId);
    }, [loadSelectedWorkerDetails, selectedWorkerId]);

    useEffect(() => {
        if (pollIntervalRef.current) {
            clearInterval(pollIntervalRef.current);
        }

        pollIntervalRef.current = setInterval(() => {
            void loadWorkers();
            if (selectedWorkerId) {
                void loadSelectedWorkerDetails(selectedWorkerId);
            }
        }, 5000);

        return () => {
            if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current);
            }
        };
    }, [loadSelectedWorkerDetails, loadWorkers, selectedWorkerId]);

    const handleRefresh = async () => {
        await loadWorkers();
        if (selectedWorkerId) {
            await loadSelectedWorkerDetails(selectedWorkerId);
        }
    };

    return (
        <div className="flex w-full gap-4 overflow-hidden">
            <aside className="w-70 min-w-70 flex flex-col bg-(--ring-sidebar-background-color,#1e1e1e) border border-(--ring-border-color,#3d3d3d) rounded overflow-hidden">
                <div className="px-4 py-3 border-b border-[var(--ring-border-color,#3d3d3d)] flex items-center justify-between">
                    <Heading level={3}>Agents</Heading>
                    <Button onClick={() => void handleRefresh()} disabled={loading}>
                        Refresh
                    </Button>
                </div>
                <div className="flex-1 overflow-y-auto">
                    {error && workers.length === 0 ? (
                        <div className="flex items-center justify-center h-full text-[var(--ring-secondary-text-color,#888)]">
                            <Text>{error}</Text>
                        </div>
                    ) : loading && workers.length === 0 ? (
                        <div className="flex items-center justify-center h-full text-[var(--ring-secondary-text-color,#888)]">
                            <Loader />
                        </div>
                    ) : workers.length === 0 ? (
                        <div className="flex items-center justify-center h-full text-[var(--ring-secondary-text-color,#888)]">
                            <Text>No agents connected</Text>
                        </div>
                    ) : (
                        <div className="p-0">
                            {workers.map((worker) => (
                                <div
                                    key={worker.id}
                                    className={`px-4 py-3 border-b border-[var(--ring-border-color,#3d3d3d)] cursor-pointer flex items-center gap-3 transition-colors hover:bg-[var(--ring-hover-background-color,#2d2d2d)] ${selectedWorkerId === worker.id ? 'bg-[var(--ring-selected-background-color,#3d3d3d)]' : ''}`}
                                    onClick={() => setSelectedWorkerId(worker.id)}
                                >
                                    <div className={`w-2.5 h-2.5 rounded-full shrink-0 ${STATUS_DOT_COLOR[worker.status]}`} />
                                    <div className="flex-1 min-w-0">
                                        <div className="font-medium overflow-hidden text-ellipsis whitespace-nowrap">{worker.name}</div>
                                        <div className="flex gap-2 text-xs text-[var(--ring-secondary-text-color,#888)] mt-0.5">
                                            <span>{worker.status}</span>
                                            <span>Tasks: {worker.current_task_count}</span>
                                            <span>{formatTimeAgo(worker.last_heartbeat)}</span>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </aside>

            <main className="flex-1 flex flex-col overflow-hidden m-4 ml-0">
                {!selectedWorkerId ? (
                    <div className="flex items-center justify-center h-full text-[var(--ring-secondary-text-color,#888)]">
                        <Text>Select an agent to view details</Text>
                    </div>
                ) : infoLoading && !selectedWorkerInfo ? (
                    <div className="flex items-center justify-center h-full text-[var(--ring-secondary-text-color,#888)]">
                        <Loader />
                    </div>
                ) : selectedWorkerInfo ? (
                    <div className="flex-1 flex flex-col gap-4 overflow-hidden">
                        {error && (
                            <div className="flex items-center justify-center h-full text-[var(--ring-secondary-text-color,#888)]">
                                <Text>{error}</Text>
                            </div>
                        )}
                        <div className="flex gap-4 shrink-0">
                            <Island>
                                <IslandHeader border>
                                    <Heading level={3}>Info</Heading>
                                </IslandHeader>
                                <IslandContent>
                                    <div className="grid grid-cols-2 gap-3">
                                        <div className="flex flex-col gap-1">
                                            <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Name</div>
                                            <div className="text-sm">{selectedWorkerInfo.name}</div>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">ID</div>
                                            <div className="text-sm">{selectedWorkerInfo.id.substring(0, 8)}...</div>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Status</div>
                                            <div className="text-sm">{selectedWorkerInfo.status}</div>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Max Concurrent</div>
                                            <div className="text-sm">{selectedWorkerInfo.max_concurrent}</div>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Has Git</div>
                                            <div className="text-sm">{selectedWorkerInfo.capabilities.has_git ? 'Yes' : 'No'}</div>
                                        </div>
                                        <div className="flex flex-col gap-1">
                                            <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Last Heartbeat</div>
                                            <div className="text-sm">{formatTimeAgo(selectedWorkerInfo.last_heartbeat)}</div>
                                        </div>
                                    </div>
                                </IslandContent>
                            </Island>

                            {selectedWorkerResources && (
                                <Island>
                                    <IslandHeader border>
                                        <Heading level={3}>Resources</Heading>
                                    </IslandHeader>
                                    <IslandContent>
                                        <div className="mb-3 last:mb-0">
                                            <div className="flex justify-between mb-1">
                                                <span>CPU</span>
                                                <span>{selectedWorkerResources.cpu_usage_percent.toFixed(1)}%</span>
                                            </div>
                                            <div className="h-2 bg-[var(--ring-border-color,#3d3d3d)] rounded overflow-hidden mt-1">
                                                <div
                                                    className={`h-full transition-[width] duration-300 ${getResourceBarColor(selectedWorkerResources.cpu_usage_percent)}`}
                                                    style={{width: `${selectedWorkerResources.cpu_usage_percent}%`}}
                                                />
                                            </div>
                                        </div>

                                        <div className="mb-3 last:mb-0">
                                            <div className="flex justify-between mb-1">
                                                <span>Memory</span>
                                                <span>
                                                    {selectedWorkerResources.memory_used_mb}MB / {selectedWorkerResources.memory_total_mb}MB (
                                                    {selectedWorkerResources.memory_usage_percent.toFixed(1)}%)
                                                </span>
                                            </div>
                                            <div className="h-2 bg-[var(--ring-border-color,#3d3d3d)] rounded overflow-hidden mt-1">
                                                <div
                                                    className={`h-full transition-[width] duration-300 ${getResourceBarColor(selectedWorkerResources.memory_usage_percent)}`}
                                                    style={{width: `${selectedWorkerResources.memory_usage_percent}%`}}
                                                />
                                            </div>
                                        </div>

                                        <div className="mb-3 last:mb-0">
                                            <div className="flex justify-between mb-1">
                                                <span>Disk</span>
                                                <span>
                                                    {selectedWorkerResources.disk_used_gb.toFixed(1)}GB / {selectedWorkerResources.disk_total_gb.toFixed(1)}GB (
                                                    {selectedWorkerResources.disk_usage_percent.toFixed(1)}%)
                                                </span>
                                            </div>
                                            <div className="h-2 bg-[var(--ring-border-color,#3d3d3d)] rounded overflow-hidden mt-1">
                                                <div
                                                    className={`h-full transition-[width] duration-300 ${getResourceBarColor(selectedWorkerResources.disk_usage_percent)}`}
                                                    style={{width: `${selectedWorkerResources.disk_usage_percent}%`}}
                                                />
                                            </div>
                                        </div>
                                    </IslandContent>
                                </Island>
                            )}

                            <Island>
                                <IslandHeader border>
                                    <Heading level={3}>Capabilities</Heading>
                                </IslandHeader>
                                <IslandContent>
                                    <div className="flex flex-col gap-1">
                                        <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Available Agents</div>
                                        <div className="flex flex-wrap gap-1.5">
                                            {selectedWorkerInfo.capabilities.available_agents.length > 0 ? (
                                                selectedWorkerInfo.capabilities.available_agents.map((agent) => <Tag key={agent}>{agent}</Tag>)
                                            ) : (
                                                <Text>None</Text>
                                            )}
                                        </div>
                                    </div>
                                    <div className="flex flex-col gap-1 mt-3">
                                        <div className="text-xs text-[var(--ring-secondary-text-color,#888)]">Supported Languages</div>
                                        <div className="flex flex-wrap gap-1.5">
                                            {selectedWorkerInfo.capabilities.supported_languages.length > 0 ? (
                                                selectedWorkerInfo.capabilities.supported_languages.map((language) => <Tag key={language}>{language}</Tag>)
                                            ) : (
                                                <Text>None</Text>
                                            )}
                                        </div>
                                    </div>
                                </IslandContent>
                            </Island>
                        </div>

                        <div className="flex-1 flex flex-col overflow-hidden min-h-0">
                            <Island className="flex-1 flex flex-col overflow-hidden">
                                <IslandHeader border>
                                    <Heading level={3}>Current Tasks</Heading>
                                    <Tag>{selectedWorkerInfo.current_tasks.length}</Tag>
                                </IslandHeader>
                                <IslandContent>
                                    {selectedWorkerInfo.current_tasks.length === 0 ? (
                                        <Text>No tasks running</Text>
                                    ) : (
                                        <div className="flex flex-wrap gap-1.5">
                                            {selectedWorkerInfo.current_tasks.map((taskId) => (
                                                <Tag key={taskId}>{taskId.substring(0, 8)}...</Tag>
                                            ))}
                                        </div>
                                    )}
                                </IslandContent>
                            </Island>
                        </div>
                    </div>
                ) : null}
            </main>
        </div>
    );
};
