/**
 * Scheduler Type Definitions
 */

export type JobType = 'skill' | 'recipe' | 'prompt' | 'system';
export type ExecutionStatus = 'running' | 'completed' | 'failed' | 'cancelled';

export interface JobConfig {
  target: string;
  params?: Record<string, any>;
}

export interface CronJob {
  id: string;
  name: string;
  schedule: string;
  jobType: JobType;
  config: JobConfig;
  enabled: boolean;
  lastRun?: string;
  nextRun?: string;
  createdAt: string;
  updatedAt: string;
}

export interface JobExecution {
  id: string;
  jobId: string;
  status: ExecutionStatus;
  result?: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
}

export interface JobCreateInput {
  id: string;
  name: string;
  schedule: string;
  jobType: JobType;
  config: JobConfig;
  enabled?: boolean;
}

export interface JobUpdateInput {
  id: string;
  name?: string;
  schedule?: string;
  config?: JobConfig;
  enabled?: boolean;
}

// Preset schedules
export const SCHEDULE_PRESETS = [
  { label: 'Every minute', value: '* * * * *' },
  { label: 'Every hour', value: '0 * * * *' },
  { label: 'Every day at midnight', value: '0 0 * * *' },
  { label: 'Every day at 9am', value: '0 9 * * *' },
  { label: 'Every week (Sunday)', value: '0 0 * * 0' },
  { label: 'Every month (1st)', value: '0 0 1 * *' },
  { label: 'Workdays at 9am', value: '0 9 * * 1-5' },
];

export const JOB_TYPE_LABELS: Record<JobType, string> = {
  skill: 'Execute Skill',
  recipe: 'Execute Recipe',
  prompt: 'Custom Prompt',
  system: 'System Task',
};
