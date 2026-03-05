import type { TaskProgressUpdate } from '@/types/task';

const STAGE_LABELS: Record<string, string> = {
  cloning: 'Cloning Repository',
  working: 'Working on Task',
  committing: 'Committing Changes',
  pushing: 'Pushing to Remote',
};

const STAGE_ICONS: Record<string, string> = {
  cloning: '📦',
  working: '🤖',
  committing: '💾',
  pushing: '🚀',
};

interface TaskProgressProps {
  progress: TaskProgressUpdate;
}

export function TaskProgress({ progress }: TaskProgressProps) {
  const percentage = progress.percentage ?? 0;
  const stageLabel = STAGE_LABELS[progress.stage] || progress.stage;
  const stageIcon = STAGE_ICONS[progress.stage] || '⚙️';

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-bold text-gray-900 mb-4">Progress</h2>

      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <span className="text-2xl">{stageIcon}</span>
            <span className="font-medium text-gray-700">{stageLabel}</span>
          </div>
          <span className="text-sm text-gray-500">{percentage}%</span>
        </div>

        <div className="w-full bg-gray-200 rounded-full h-3">
          <div
            className="bg-blue-600 h-3 rounded-full transition-all duration-300"
            style={{ width: `${percentage}%` }}
          />
        </div>
      </div>

      <div className="bg-gray-50 rounded p-3">
        <p className="text-sm text-gray-600">{progress.message}</p>
      </div>
    </div>
  );
}
