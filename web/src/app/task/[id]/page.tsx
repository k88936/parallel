'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import { useWebSocket } from '@/hooks/useWebSocket';
import { TaskProgress } from '@/components/TaskProgress';
import { AgentOutput } from '@/components/AgentOutput';
import { HumanInteraction } from '@/components/HumanInteraction';
import { api } from '@/lib/api';
import type { Task, HumanNotification, TaskProgressUpdate } from '@/types/task';

export default function TaskDetailPage() {
  const params = useParams();
  const taskId = params.id as string;

  const [task, setTask] = useState<Task | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [output, setOutput] = useState<string[]>([]);
  const [progress, setProgress] = useState<TaskProgressUpdate | null>(null);

  useEffect(() => {
    const fetchTask = async () => {
      try {
        setLoading(true);
        const taskData = await api.getTask(taskId);
        setTask(taskData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch task');
      } finally {
        setLoading(false);
      }
    };

    fetchTask();
  }, [taskId]);

  const handleNotification = (notification: HumanNotification) => {
    switch (notification.type) {
      case 'task_progress':
        setProgress(notification.update);
        break;
      case 'agent_output':
        setOutput((prev) => [...prev, notification.output]);
        break;
      case 'terminal_output':
        setOutput((prev) => [...prev, `[Terminal] ${notification.output}`]);
        break;
      case 'task_status_update':
        if (task) {
          setTask({ ...task, status: notification.status });
        }
        break;
      case 'task_completed':
        setOutput((prev) => [...prev, `\n✓ Task completed! Branch: ${notification.branch}`]);
        break;
      case 'task_awaiting_review':
        setOutput((prev) => [...prev, '\n⏸ Task awaiting review...']);
        break;
    }
  };

  const { isConnected, sendMessage, isReconnecting } = useWebSocket({
    taskId,
    onMessage: handleNotification,
    onConnect: () => console.log('Connected to task', taskId),
    onDisconnect: () => console.log('Disconnected from task', taskId),
  });

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-xl">Loading task...</div>
      </div>
    );
  }

  if (error || !task) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-xl text-red-600">
          Error: {error || 'Task not found'}
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <div className="container mx-auto px-4 py-8 max-w-7xl">
        <header className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">
                Task: {task.description}
              </h1>
              <p className="text-gray-600 mt-2">
                Repository: {task.repo_url}
              </p>
            </div>
            <div className="flex items-center gap-4">
              <div
                className={`px-3 py-1 rounded text-sm font-medium ${
                  isConnected
                    ? 'bg-green-100 text-green-800'
                    : isReconnecting
                    ? 'bg-yellow-100 text-yellow-800'
                    : 'bg-red-100 text-red-800'
                }`}
              >
                {isConnected ? 'Connected' : isReconnecting ? 'Reconnecting...' : 'Disconnected'}
              </div>
            </div>
          </div>
        </header>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            {progress && <TaskProgress progress={progress} />}
            <AgentOutput output={output} />
          </div>

          <div>
            <HumanInteraction
              taskId={taskId}
              isConnected={isConnected}
              sendMessage={sendMessage}
              taskStatus={task.status}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
