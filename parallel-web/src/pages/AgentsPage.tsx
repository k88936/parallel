import {useCallback, useEffect, useRef, useState} from 'react';
import {workersApi} from '../api';
import type {ResourceMonitor, WorkerInfo, WorkerStatus, WorkerSummary} from '../types';
import {Sidebar} from '../components/Layout';

import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Tag from '@jetbrains/ring-ui-built/components/tag/tag';
import Group from "@jetbrains/ring-ui-built/components/group/group";

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

    return (
        <Group className="flex w-full gap-4">
            <Sidebar
                title="Agents"
            >
                {error && workers.length === 0 ? (
                    <Text>{error}</Text>
                ) : loading && workers.length === 0 ? (
                    <Loader/>
                ) : workers.length === 0 ? (
                    <Text>No agents connected</Text>
                ) : (
                    <Group className="p-0">
                        {workers.map((worker) => (
                            <Group
                                key={worker.id}
                                className={`px-4 py-3 rounded-2xl cursor-pointer flex items-center gap-3 hover:bg-(--ring-hover-background-color) ${selectedWorkerId === worker.id ? 'bg-(--ring-selected-background-color)' : ''}`}
                                onClick={() => setSelectedWorkerId(worker.id)}
                            >
                                <Group
                                    className={`w-2.5 h-2.5 rounded-full shrink-0 ${STATUS_DOT_COLOR[worker.status]}`}/>
                                <Group className="flex-1 min-w-0">
                                    <Group
                                        className="font-medium overflow-hidden text-ellipsis whitespace-nowrap">{worker.name}</Group>
                                    <Group className="flex gap-2 text-xs text-(--ring-secondary-text-color) mt-0.5">
                                        <Group>{worker.status}</Group>
                                        <Group>Tasks: {worker.current_task_count}</Group>
                                        <Group>{formatTimeAgo(worker.last_heartbeat)}</Group>
                                    </Group>
                                </Group>
                            </Group>
                        ))}
                    </Group>
                )}
            </Sidebar>

            <Group className="flex-1 flex flex-col">
                {!selectedWorkerId ? (
                    <Group
                        className="flex items-center justify-center flex-1">
                        <Text>Select an agent to view details</Text>
                    </Group>
                ) : infoLoading && !selectedWorkerInfo ? (
                    <Group
                        className="flex items-center justify-center flex-1">
                        <Loader/>
                    </Group>
                ) : selectedWorkerInfo ? (
                    <Group className="flex-1 flex flex-col gap-4 overflow-hidden">
                        {error && (
                            <Group
                                className="flex items-center justify-center h-full">
                                <Text>{error}</Text>
                            </Group>
                        )}
                        <Group className="flex gap-4 shrink-0">
                            <Island className={"flex-1"}>
                                <IslandHeader border>
                                    <Heading level={3}>Info</Heading>
                                </IslandHeader>
                                <IslandContent>
                                    <Group className="grid grid-cols-2 gap-3">
                                        <Group className="flex flex-col gap-1">
                                            <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">Name
                                            </Group>
                                            <Group className="text-sm">{selectedWorkerInfo.name}</Group>
                                        </Group>
                                        <Group className="flex flex-col gap-1">
                                            <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">ID
                                            </Group>
                                            <Group className="text-sm">{selectedWorkerInfo.id.substring(0, 8)}...</Group>
                                        </Group>
                                        <Group className="flex flex-col gap-1">
                                            <Group
                                                className="text-xs text-[var(--ring-secondary-text-color,#888)]">Status
                                            </Group>
                                            <Group className="text-sm">{selectedWorkerInfo.status}</Group>
                                        </Group>
                                        <Group className="flex flex-col gap-1">
                                            <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">Max
                                                Concurrent
                                            </Group>
                                            <Group className="text-sm">{selectedWorkerInfo.max_concurrent}</Group>
                                        </Group>
                                        <Group className="flex flex-col gap-1">
                                            <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">Has
                                                Git
                                            </Group>
                                            <Group
                                                className="text-sm">{selectedWorkerInfo.capabilities.has_git ? 'Yes' : 'No'}</Group>
                                        </Group>
                                        <Group className="flex flex-col gap-1">
                                            <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">Last
                                                Heartbeat
                                            </Group>
                                            <Group
                                                className="text-sm">{formatTimeAgo(selectedWorkerInfo.last_heartbeat)}</Group>
                                        </Group>
                                    </Group>
                                </IslandContent>
                            </Island>

                            {selectedWorkerResources && (
                                <Island className={"flex-1"}>
                                    <IslandHeader border>
                                        <Heading level={3}>Resources</Heading>
                                    </IslandHeader>
                                    <IslandContent>
                                        <Group className="mb-3 last:mb-0">
                                            <Group className="flex justify-between mb-1">
                                                <Text>CPU</Text>
                                                <Text>{selectedWorkerResources.cpu_usage_percent.toFixed(1)}%</Text>
                                            </Group>
                                            <Group
                                                className="h-2 bg-[var(--ring-border-color,#3d3d3d)] rounded overflow-hidden mt-1">
                                                <Group
                                                    className={`h-full transition-[width] duration-300 ${getResourceBarColor(selectedWorkerResources.cpu_usage_percent)}`}
                                                    style={{width: `${selectedWorkerResources.cpu_usage_percent}%`}}
                                                />
                                            </Group>
                                        </Group>

                                        <Group className="mb-3 last:mb-0">
                                            <Group className="flex justify-between mb-1">
                                                <Text>Memory</Text>
                                                <Text>
                                                    {selectedWorkerResources.memory_used_mb}MB / {selectedWorkerResources.memory_total_mb}MB (
                                                    {selectedWorkerResources.memory_usage_percent.toFixed(1)}%)
                                                </Text>
                                            </Group>
                                            <Group
                                                className="h-2 bg-[var(--ring-border-color,#3d3d3d)] rounded overflow-hidden mt-1">
                                                <Group
                                                    className={`h-full transition-[width] duration-300 ${getResourceBarColor(selectedWorkerResources.memory_usage_percent)}`}
                                                    style={{width: `${selectedWorkerResources.memory_usage_percent}%`}}
                                                />
                                            </Group>
                                        </Group>

                                        <Group className="mb-3 last:mb-0">
                                            <Group className="flex justify-between mb-1">
                                                <Text>Disk</Text>
                                                <Text>
                                                    {selectedWorkerResources.disk_used_gb.toFixed(1)}GB / {selectedWorkerResources.disk_total_gb.toFixed(1)}GB (
                                                    {selectedWorkerResources.disk_usage_percent.toFixed(1)}%)
                                                </Text>
                                            </Group>
                                            <Group
                                                className="h-2 bg-[var(--ring-border-color,#3d3d3d)] rounded overflow-hidden mt-1">
                                                <Group
                                                    className={`h-full transition-[width] duration-300 ${getResourceBarColor(selectedWorkerResources.disk_usage_percent)}`}
                                                    style={{width: `${selectedWorkerResources.disk_usage_percent}%`}}
                                                />
                                            </Group>
                                        </Group>
                                    </IslandContent>
                                </Island>
                            )}

                            <Island className={"flex-1"}>
                                <IslandHeader border>
                                    <Heading level={3}>Capabilities</Heading>
                                </IslandHeader>
                                <IslandContent>
                                    <Group className="flex flex-col gap-1">
                                        <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">Available
                                            Agents
                                        </Group>
                                        <Group className="flex flex-wrap gap-1.5">
                                            {selectedWorkerInfo.capabilities.available_agents.length > 0 ? (
                                                selectedWorkerInfo.capabilities.available_agents.map((agent) => <Tag
                                                    key={agent}>{agent}</Tag>)
                                            ) : (
                                                <Text>None</Text>
                                            )}
                                        </Group>
                                    </Group>
                                    <Group className="flex flex-col gap-1 mt-3">
                                        <Group className="text-xs text-[var(--ring-secondary-text-color,#888)]">Supported
                                            Languages
                                        </Group>
                                        <Group className="flex flex-wrap gap-1.5">
                                            {selectedWorkerInfo.capabilities.supported_languages.length > 0 ? (
                                                selectedWorkerInfo.capabilities.supported_languages.map((language) =>
                                                    <Tag key={language}>{language}</Tag>)
                                            ) : (
                                                <Text>None</Text>
                                            )}
                                        </Group>
                                    </Group>
                                </IslandContent>
                            </Island>
                        </Group>

                        <Group className="flex-1 flex flex-col overflow-hidden min-h-0">
                            <Island className="flex-1 flex flex-col overflow-hidden">
                                <IslandHeader border>
                                    <Heading level={3}>Current Tasks</Heading>
                                    <Tag>{selectedWorkerInfo.current_tasks.length}</Tag>
                                </IslandHeader>
                                <IslandContent>
                                    {selectedWorkerInfo.current_tasks.length === 0 ? (
                                        <Text>No tasks running</Text>
                                    ) : (
                                        <Group className="flex flex-wrap gap-1.5">
                                            {selectedWorkerInfo.current_tasks.map((taskId) => (
                                                <Tag key={taskId}>{taskId.substring(0, 8)}...</Tag>
                                            ))}
                                        </Group>
                                    )}
                                </IslandContent>
                            </Island>
                        </Group>
                    </Group>
                ) : null}
            </Group>
        </Group>
    );
};
