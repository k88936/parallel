import { useEffect, useRef } from 'react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
    fetchWorkers,
    fetchWorkerInfo,
    fetchWorkerResources,
    selectWorker,
} from '../store/slices/workersSlice';
import type { WorkerStatus } from '../types';
import styles from './AgentsPage.module.css';

import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Loader from '@jetbrains/ring-ui-built/components/loader/loader';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Tag from '@jetbrains/ring-ui-built/components/tag/tag';

const getStatusColor = (status: WorkerStatus): string => {
    switch (status) {
        case 'idle':
            return styles.idle;
        case 'busy':
            return styles.busy;
        case 'offline':
            return styles.offline;
        case 'dead':
            return styles.dead;
        default:
            return '';
    }
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

const getResourceLevel = (percent: number): string => {
    if (percent < 50) return styles.low;
    if (percent < 80) return styles.medium;
    return styles.high;
};

export const AgentsPage = () => {
    const dispatch = useAppDispatch();
    const {
        workers,
        selectedWorkerId,
        selectedWorkerInfo,
        selectedWorkerResources,
        loading,
        infoLoading,
    } = useAppSelector((state) => state.workers);

    const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

    useEffect(() => {
        dispatch(fetchWorkers());

        pollIntervalRef.current = setInterval(() => {
            dispatch(fetchWorkers());
        }, 5000);

        return () => {
            if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current);
            }
        };
    }, [dispatch]);

    useEffect(() => {
        if (selectedWorkerId) {
            dispatch(fetchWorkerInfo(selectedWorkerId));
            dispatch(fetchWorkerResources(selectedWorkerId));
        }
    }, [selectedWorkerId, dispatch]);

    useEffect(() => {
        if (pollIntervalRef.current && selectedWorkerId) {
            clearInterval(pollIntervalRef.current);
            pollIntervalRef.current = setInterval(() => {
                dispatch(fetchWorkers());
                dispatch(fetchWorkerInfo(selectedWorkerId));
                dispatch(fetchWorkerResources(selectedWorkerId));
            }, 5000);
        }
    }, [selectedWorkerId, dispatch]);

    const handleWorkerClick = (workerId: string) => {
        dispatch(selectWorker(workerId));
    };

    const handleRefresh = () => {
        dispatch(fetchWorkers());
        if (selectedWorkerId) {
            dispatch(fetchWorkerInfo(selectedWorkerId));
            dispatch(fetchWorkerResources(selectedWorkerId));
        }
    };

    return (
        <div className={styles.container}>
            <aside className={styles.sidebar}>
                <div className={styles.sidebarHeader}>
                    <Heading level={3}>Agents</Heading>
                    <Button onClick={handleRefresh} disabled={loading}>
                        Refresh
                    </Button>
                </div>
                <div className={styles.sidebarContent}>
                    {loading && workers.length === 0 ? (
                        <div className={styles.empty}>
                            <Loader />
                        </div>
                    ) : workers.length === 0 ? (
                        <div className={styles.empty}>
                            <Text>No agents connected</Text>
                        </div>
                    ) : (
                        <div className={styles.workerList}>
                            {workers.map((worker) => (
                                <div
                                    key={worker.id}
                                    className={`${styles.workerItem} ${
                                        selectedWorkerId === worker.id
                                            ? styles.selected
                                            : ''
                                    }`}
                                    onClick={() => handleWorkerClick(worker.id)}
                                >
                                    <div
                                        className={`${styles.statusDot} ${getStatusColor(
                                            worker.status
                                        )}`}
                                    />
                                    <div className={styles.workerInfo}>
                                        <div className={styles.workerName}>
                                            {worker.name}
                                        </div>
                                        <div className={styles.workerMeta}>
                                            <span>{worker.status}</span>
                                            <span>
                                                Tasks: {worker.current_task_count}
                                            </span>
                                            <span>
                                                {formatTimeAgo(worker.last_heartbeat)}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </aside>

            <main className={styles.main}>
                {!selectedWorkerId ? (
                    <div className={styles.empty}>
                        <Text>Select an agent to view details</Text>
                    </div>
                ) : infoLoading && !selectedWorkerInfo ? (
                    <div className={styles.empty}>
                        <Loader />
                    </div>
                ) : selectedWorkerInfo ? (
                    <div className={styles.detailContent}>
                        <div className={styles.topRow}>
                            <Island>
                                <IslandHeader border>
                                    <Heading level={3}>Info</Heading>
                                </IslandHeader>
                                <IslandContent>
                                    <div className={styles.infoGrid}>
                                        <div className={styles.infoItem}>
                                            <div className={styles.infoLabel}>Name</div>
                                            <div className={styles.infoValue}>
                                                {selectedWorkerInfo.name}
                                            </div>
                                        </div>
                                        <div className={styles.infoItem}>
                                            <div className={styles.infoLabel}>ID</div>
                                            <div className={styles.infoValue}>
                                                {selectedWorkerInfo.id.substring(0, 8)}...
                                            </div>
                                        </div>
                                        <div className={styles.infoItem}>
                                            <div className={styles.infoLabel}>Status</div>
                                            <div className={styles.infoValue}>
                                                {selectedWorkerInfo.status}
                                            </div>
                                        </div>
                                        <div className={styles.infoItem}>
                                            <div className={styles.infoLabel}>
                                                Max Concurrent
                                            </div>
                                            <div className={styles.infoValue}>
                                                {selectedWorkerInfo.max_concurrent}
                                            </div>
                                        </div>
                                        <div className={styles.infoItem}>
                                            <div className={styles.infoLabel}>Has Git</div>
                                            <div className={styles.infoValue}>
                                                {selectedWorkerInfo.capabilities.has_git
                                                    ? 'Yes'
                                                    : 'No'}
                                            </div>
                                        </div>
                                        <div className={styles.infoItem}>
                                            <div className={styles.infoLabel}>Last Heartbeat</div>
                                            <div className={styles.infoValue}>
                                                {formatTimeAgo(selectedWorkerInfo.last_heartbeat)}
                                            </div>
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
                                        <div className={styles.resourceRow}>
                                            <div className={styles.resourceLabel}>
                                                <span>CPU</span>
                                                <span>
                                                    {selectedWorkerResources.cpu_usage_percent.toFixed(
                                                        1
                                                    )}
                                                    %
                                                </span>
                                            </div>
                                            <div className={styles.resourceBar}>
                                                <div
                                                    className={`${styles.resourceBarFill} ${getResourceLevel(
                                                        selectedWorkerResources.cpu_usage_percent
                                                    )}`}
                                                    style={{
                                                        width: `${selectedWorkerResources.cpu_usage_percent}%`,
                                                    }}
                                                />
                                            </div>
                                        </div>

                                        <div className={styles.resourceRow}>
                                            <div className={styles.resourceLabel}>
                                                <span>Memory</span>
                                                <span>
                                                    {selectedWorkerResources.memory_used_mb}MB /{' '}
                                                    {selectedWorkerResources.memory_total_mb}MB (
                                                    {selectedWorkerResources.memory_usage_percent.toFixed(
                                                        1
                                                    )}
                                                    %)
                                                </span>
                                            </div>
                                            <div className={styles.resourceBar}>
                                                <div
                                                    className={`${styles.resourceBarFill} ${getResourceLevel(
                                                        selectedWorkerResources.memory_usage_percent
                                                    )}`}
                                                    style={{
                                                        width: `${selectedWorkerResources.memory_usage_percent}%`,
                                                    }}
                                                />
                                            </div>
                                        </div>

                                        <div className={styles.resourceRow}>
                                            <div className={styles.resourceLabel}>
                                                <span>Disk</span>
                                                <span>
                                                    {selectedWorkerResources.disk_used_gb.toFixed(
                                                        1
                                                    )}
                                                    GB /{' '}
                                                    {selectedWorkerResources.disk_total_gb.toFixed(
                                                        1
                                                    )}
                                                    GB (
                                                    {selectedWorkerResources.disk_usage_percent.toFixed(
                                                        1
                                                    )}
                                                    %)
                                                </span>
                                            </div>
                                            <div className={styles.resourceBar}>
                                                <div
                                                    className={`${styles.resourceBarFill} ${getResourceLevel(
                                                        selectedWorkerResources.disk_usage_percent
                                                    )}`}
                                                    style={{
                                                        width: `${selectedWorkerResources.disk_usage_percent}%`,
                                                    }}
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
                                    <div className={styles.infoItem}>
                                        <div className={styles.infoLabel}>Available Agents</div>
                                        <div className={styles.capabilityList}>
                                            {selectedWorkerInfo.capabilities.available_agents
                                                .length > 0 ? (
                                                selectedWorkerInfo.capabilities.available_agents.map(
                                                    (agent) => <Tag key={agent}>{agent}</Tag>
                                                )
                                            ) : (
                                                <Text>None</Text>
                                            )}
                                        </div>
                                    </div>
                                    <div className={styles.infoItem} style={{ marginTop: 12 }}>
                                        <div className={styles.infoLabel}>Supported Languages</div>
                                        <div className={styles.capabilityList}>
                                            {selectedWorkerInfo.capabilities.supported_languages
                                                .length > 0 ? (
                                                selectedWorkerInfo.capabilities.supported_languages.map(
                                                    (lang) => <Tag key={lang}>{lang}</Tag>
                                                )
                                            ) : (
                                                <Text>None</Text>
                                            )}
                                        </div>
                                    </div>
                                </IslandContent>
                            </Island>
                        </div>

                        <div className={styles.tasksSection}>
                            <Island className={styles.tasksIsland}>
                                <IslandHeader border>
                                    <Heading level={3}>Current Tasks</Heading>
                                    <Tag>{selectedWorkerInfo.current_tasks.length}</Tag>
                                </IslandHeader>
                                <IslandContent>
                                    {selectedWorkerInfo.current_tasks.length === 0 ? (
                                        <Text>No tasks running</Text>
                                    ) : (
                                        <div className={styles.capabilityList}>
                                            {selectedWorkerInfo.current_tasks.map((taskId) => (
                                                <Tag key={taskId}>
                                                    {taskId.substring(0, 8)}...
                                                </Tag>
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
