/**
 * Job List Component - Cron job management
 */

import React, { useState, useEffect } from 'react';
import { Plus, Trash2, Play, Clock, ToggleLeft, ToggleRight } from 'lucide-react';
import { useSchedulerStore } from '../../stores/schedulerStore';
import type { CronJob, JobType } from '../../types/scheduler';
import { JOB_TYPE_LABELS, SCHEDULE_PRESETS } from '../../types/scheduler';

export function JobList() {
  const { jobs, loading, error, loadJobs, createJob, deleteJob, toggleJob, runJobNow } = useSchedulerStore();
  const [showCreate, setShowCreate] = useState(false);
  const [newJob, setNewJob] = useState({
    name: '',
    schedule: '0 * * * *',
    jobType: 'prompt' as JobType,
    target: '',
  });

  useEffect(() => {
    loadJobs();
  }, [loadJobs]);

  const handleCreate = async () => {
    if (!newJob.name || !newJob.target) return;

    await createJob({
      id: `job-${Date.now()}`,
      name: newJob.name,
      schedule: newJob.schedule,
      jobType: newJob.jobType,
      config: { target: newJob.target },
      enabled: true,
    });

    setNewJob({ name: '', schedule: '0 * * * *', jobType: 'prompt', target: '' });
    setShowCreate(false);
  };

  const handleRunNow = async (id: string) => {
    try {
      const execId = await runJobNow(id);
      console.log('Job started:', execId);
    } catch (err) {
      console.error('Failed to run job:', err);
    }
  };

  const getStatusColor = (enabled: boolean) => {
    return enabled ? 'bg-green-100 text-green-600' : 'bg-gray-100 text-gray-400';
  };

  if (loading) {
    return <div className="p-4 text-center text-gray-500">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      {error && (
        <div className="p-3 bg-red-50 text-red-600 rounded text-sm">{error}</div>
      )}

      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Scheduled Jobs</h2>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="flex items-center gap-1 px-3 py-1.5 bg-blue-500 text-white rounded text-sm hover:bg-blue-600"
        >
          <Plus className="w-4 h-4" />
          New Job
        </button>
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="p-4 border rounded-lg bg-gray-50 space-y-3">
          <div>
            <label className="block text-sm font-medium mb-1">Job Name</label>
            <input
              type="text"
              value={newJob.name}
              onChange={(e) => setNewJob({ ...newJob, name: e.target.value })}
              className="w-full px-3 py-2 border rounded text-sm"
              placeholder="Daily summary"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Schedule</label>
            <select
              value={newJob.schedule}
              onChange={(e) => setNewJob({ ...newJob, schedule: e.target.value })}
              className="w-full px-3 py-2 border rounded text-sm"
            >
              {SCHEDULE_PRESETS.map((preset) => (
                <option key={preset.value} value={preset.value}>
                  {preset.label}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Job Type</label>
            <select
              value={newJob.jobType}
              onChange={(e) => setNewJob({ ...newJob, jobType: e.target.value as JobType })}
              className="w-full px-3 py-2 border rounded text-sm"
            >
              {Object.entries(JOB_TYPE_LABELS).map(([value, label]) => (
                <option key={value} value={value}>{label}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Target (Skill/Recipe ID or Prompt)</label>
            <textarea
              value={newJob.target}
              onChange={(e) => setNewJob({ ...newJob, target: e.target.value })}
              className="w-full px-3 py-2 border rounded text-sm"
              rows={3}
              placeholder={newJob.jobType === 'prompt' ? 'Enter prompt...' : 'Enter skill or recipe ID'}
            />
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleCreate}
              className="flex-1 px-4 py-2 bg-blue-500 text-white rounded text-sm hover:bg-blue-600"
            >
              Create Job
            </button>
            <button
              onClick={() => setShowCreate(false)}
              className="px-4 py-2 border rounded text-sm hover:bg-gray-100"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Job List */}
      {jobs.length === 0 ? (
        <div className="p-8 text-center text-gray-400">
          No scheduled jobs. Click "New Job" to create one.
        </div>
      ) : (
        <div className="space-y-2">
          {jobs.map((job) => (
            <div
              key={job.id}
              className={`p-4 border rounded-lg ${!job.enabled ? 'opacity-60' : ''}`}
            >
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-medium">{job.name}</h3>
                    <span className={`px-2 py-0.5 text-xs rounded ${getStatusColor(job.enabled)}`}>
                      {job.enabled ? 'Active' : 'Paused'}
                    </span>
                  </div>
                  <div className="flex items-center gap-3 mt-1 text-sm text-gray-500">
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {job.schedule}
                    </span>
                    <span>{JOB_TYPE_LABELS[job.jobType]}</span>
                  </div>
                  {job.lastRun && (
                    <div className="text-xs text-gray-400 mt-1">
                      Last run: {new Date(job.lastRun).toLocaleString()}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleRunNow(job.id)}
                    className="p-2 hover:bg-blue-100 rounded text-blue-500"
                    title="Run now"
                  >
                    <Play className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => toggleJob(job.id)}
                    className={`p-2 rounded ${job.enabled ? 'text-green-500' : 'text-gray-400'}`}
                    title={job.enabled ? 'Disable' : 'Enable'}
                  >
                    {job.enabled ? (
                      <ToggleRight className="w-5 h-5" />
                    ) : (
                      <ToggleLeft className="w-5 h-5" />
                    )}
                  </button>
                  <button
                    onClick={() => deleteJob(job.id)}
                    className="p-2 hover:bg-red-100 rounded text-red-500"
                    title="Delete"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default JobList;
