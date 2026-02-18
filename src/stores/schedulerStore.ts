/**
 * Scheduler Store - Zustand state management for cron jobs
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { CronJob, JobExecution, JobCreateInput, JobUpdateInput, JobType } from '../types/scheduler';

interface SchedulerState {
  jobs: CronJob[];
  executions: JobExecution[];
  loading: boolean;
  error: string | null;

  // Actions
  loadJobs: () => Promise<void>;
  createJob: (job: JobCreateInput) => Promise<void>;
  updateJob: (job: JobUpdateInput) => Promise<void>;
  deleteJob: (id: string) => Promise<void>;
  toggleJob: (id: string) => Promise<void>;
  runJobNow: (id: string) => Promise<string>;
  loadExecutions: (jobId?: string) => Promise<void>;
}

export const useSchedulerStore = create<SchedulerState>((set, get) => ({
  jobs: [],
  executions: [],
  loading: false,
  error: null,

  loadJobs: async () => {
    set({ loading: true, error: null });
    try {
      const rawJobs = await invoke<Array<{
        id: string;
        name: string;
        schedule: string;
        job_type: string;
        config: string;
        enabled: number;
        last_run: string | null;
        next_run: string | null;
        created_at: string;
        updated_at: string;
      }>>('list_cron_jobs');

      const jobs: CronJob[] = rawJobs.map((j) => ({
        id: j.id,
        name: j.name,
        schedule: j.schedule,
        jobType: j.job_type as JobType,
        config: JSON.parse(j.config || '{}'),
        enabled: j.enabled !== 0,
        lastRun: j.last_run || undefined,
        nextRun: j.next_run || undefined,
        createdAt: j.created_at,
        updatedAt: j.updated_at,
      }));

      set({ jobs, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createJob: async (input: JobCreateInput) => {
    try {
      await invoke('create_cron_job', {
        id: input.id,
        name: input.name,
        schedule: input.schedule,
        jobType: input.jobType,
        config: JSON.stringify(input.config),
        enabled: input.enabled !== false ? 1 : 0,
      });

      await get().loadJobs();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateJob: async (input: JobUpdateInput) => {
    try {
      await invoke('update_cron_job', {
        id: input.id,
        name: input.name,
        schedule: input.schedule,
        config: input.config ? JSON.stringify(input.config) : null,
        enabled: input.enabled !== undefined ? (input.enabled ? 1 : 0) : null,
      });

      await get().loadJobs();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteJob: async (id: string) => {
    try {
      await invoke('delete_cron_job', { id });

      set((state) => ({
        jobs: state.jobs.filter((j) => j.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  toggleJob: async (id: string) => {
    const job = get().jobs.find((j) => j.id === id);
    if (job) {
      await get().updateJob({ id, enabled: !job.enabled });
    }
  },

  runJobNow: async (id: string) => {
    const executionId = await invoke<string>('run_cron_job_now', { id });
    return executionId;
  },

  loadExecutions: async (jobId?: string) => {
    try {
      const rawExecutions = await invoke<Array<{
        id: string;
        job_id: string;
        status: string;
        result: string | null;
        error: string | null;
        started_at: string;
        completed_at: string | null;
      }>>('list_job_executions', { jobId: jobId || null });

      const executions: JobExecution[] = rawExecutions.map((e) => ({
        id: e.id,
        jobId: e.job_id,
        status: e.status as JobExecution['status'],
        result: e.result || undefined,
        error: e.error || undefined,
        startedAt: e.started_at,
        completedAt: e.completed_at || undefined,
      }));

      set({ executions });
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
