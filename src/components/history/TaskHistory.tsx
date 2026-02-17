/**
 * Task History Component
 */

import React from 'react';
import { Clock, CheckCircle, XCircle, Loader2, ChevronRight } from 'lucide-react';

interface Task {
  id: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  startTime: Date;
  endTime?: Date;
  result?: string;
  error?: string;
}

// Mock tasks for development
const mockTasks: Task[] = [
  {
    id: '1',
    description: 'Organize Downloads folder',
    status: 'completed',
    startTime: new Date(Date.now() - 3600000),
    endTime: new Date(Date.now() - 3500000),
    result: 'Moved 12 files to sorted folders',
  },
  {
    id: '2',
    description: 'Generate report summary',
    status: 'completed',
    startTime: new Date(Date.now() - 7200000),
    endTime: new Date(Date.now() - 7100000),
    result: 'Created summary.md with 5 sections',
  },
  {
    id: '3',
    description: 'Analyze codebase structure',
    status: 'running',
    startTime: new Date(Date.now() - 60000),
  },
];

export function TaskHistory() {
  const [tasks] = React.useState<Task[]>(mockTasks);

  const groupedTasks = groupTasksByDate(tasks);

  if (tasks.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500">
        <div className="text-center">
          <Clock className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <h2 className="text-lg font-medium">No Task History</h2>
          <p className="text-sm">Your completed tasks will appear here</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b">
        <h2 className="font-semibold text-lg">Task History</h2>
        <p className="text-sm text-gray-500">View your recent AI tasks</p>
      </div>

      {/* Task List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {Object.entries(groupedTasks).map(([date, dateTasks]) => (
          <div key={date}>
            <h3 className="text-sm font-medium text-gray-500 mb-2">{date}</h3>
            <div className="space-y-2">
              {dateTasks.map((task) => (
                <TaskItem key={task.id} task={task} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

interface TaskItemProps {
  task: Task;
}

function TaskItem({ task }: TaskItemProps) {
  const [expanded, setExpanded] = React.useState(false);

  const StatusIcon = {
    pending: Clock,
    running: Loader2,
    completed: CheckCircle,
    failed: XCircle,
  }[task.status];

  const statusColor = {
    pending: 'text-gray-400',
    running: 'text-blue-500',
    completed: 'text-green-500',
    failed: 'text-red-500',
  }[task.status];

  const duration = task.endTime
    ? Math.round((task.endTime.getTime() - task.startTime.getTime()) / 1000)
    : null;

  return (
    <div className="border rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between p-3 hover:bg-gray-50 dark:hover:bg-gray-800 text-left"
      >
        <div className="flex items-center gap-3">
          <StatusIcon className={`w-5 h-5 ${statusColor} ${task.status === 'running' ? 'animate-spin' : ''}`} />
          <div>
            <p className="font-medium text-sm">{task.description}</p>
            <p className="text-xs text-gray-500">
              {task.startTime.toLocaleTimeString()}
              {duration !== null && ` · ${duration}s`}
            </p>
          </div>
        </div>
        <ChevronRight className={`w-4 h-4 text-gray-400 transition-transform ${expanded ? 'rotate-90' : ''}`} />
      </button>

      {expanded && (
        <div className="px-3 pb-3 pt-0 border-t text-sm">
          {task.status === 'completed' && task.result && (
            <div className="mt-2 p-2 bg-green-50 dark:bg-green-900/20 rounded text-green-700 dark:text-green-300">
              {task.result}
            </div>
          )}
          {task.status === 'failed' && task.error && (
            <div className="mt-2 p-2 bg-red-50 dark:bg-red-900/20 rounded text-red-700 dark:text-red-300">
              {task.error}
            </div>
          )}
          {task.status === 'running' && (
            <div className="mt-2 p-2 bg-blue-50 dark:bg-blue-900/20 rounded text-blue-700 dark:text-blue-300">
              Task in progress...
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function groupTasksByDate(tasks: Task[]): Record<string, Task[]> {
  const groups: Record<string, Task[]> = {};
  const today = new Date().toDateString();
  const yesterday = new Date(Date.now() - 86400000).toDateString();

  // Sort tasks by start time (newest first)
  const sorted = [...tasks].sort((a, b) => b.startTime.getTime() - a.startTime.getTime());

  for (const task of sorted) {
    const date = task.startTime.toDateString();
    let label: string;

    if (date === today) {
      label = 'Today';
    } else if (date === yesterday) {
      label = 'Yesterday';
    } else {
      label = task.startTime.toLocaleDateString(undefined, {
        weekday: 'long',
        month: 'short',
        day: 'numeric',
      });
    }

    if (!groups[label]) {
      groups[label] = [];
    }
    groups[label].push(task);
  }

  return groups;
}
