/**
 * SubAgent List Component
 */

import React, { useState, useEffect } from 'react';
import { Plus, Trash2, Play, Pause, RotateCcw } from 'lucide-react';
import { useSubAgentStore } from '../../stores/subAgentStore';
import type { SubAgent, SubAgentType, SubAgentCreateInput } from '../../types/subagent';
import { AGENT_TYPE_LABELS, AGENT_STATUS_COLORS } from '../../types/subagent';

export function SubAgentList() {
  const { agents, loading, error, loadAgents, createAgent, deleteAgent, assignTask } = useSubAgentStore();
  const [showCreate, setShowCreate] = useState(false);
  const [newAgent, setNewAgent] = useState<Partial<SubAgentCreateInput>>({
    type: 'executor',
    name: '',
  });

  useEffect(() => {
    loadAgents();
  }, [loadAgents]);

  const handleCreate = async () => {
    if (!newAgent.name || !newAgent.type) return;

    await createAgent({
      id: `agent-${Date.now()}`,
      name: newAgent.name,
      type: newAgent.type as SubAgentType,
    });

    setNewAgent({ type: 'executor', name: '' });
    setShowCreate(false);
  };

  const getStatusBadge = (status: SubAgent['status']) => {
    const colors: Record<string, string> = {
      idle: 'bg-gray-100 text-gray-600',
      running: 'bg-blue-100 text-blue-600',
      paused: 'bg-yellow-100 text-yellow-600',
      completed: 'bg-green-100 text-green-600',
      failed: 'bg-red-100 text-red-600',
    };
    return (
      <span className={`px-2 py-0.5 text-xs rounded ${colors[status]}`}>
        {status}
      </span>
    );
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
        <h2 className="text-lg font-semibold">Sub-agents</h2>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="flex items-center gap-1 px-3 py-1.5 bg-blue-500 text-white rounded text-sm hover:bg-blue-600"
        >
          <Plus className="w-4 h-4" />
          Create
        </button>
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="p-4 border rounded-lg bg-gray-50 space-y-3">
          <div>
            <label className="block text-sm font-medium mb-1">Name</label>
            <input
              type="text"
              value={newAgent.name || ''}
              onChange={(e) => setNewAgent({ ...newAgent, name: e.target.value })}
              className="w-full px-3 py-2 border rounded text-sm"
              placeholder="Agent name"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Type</label>
            <select
              value={newAgent.type || 'executor'}
              onChange={(e) => setNewAgent({ ...newAgent, type: e.target.value as SubAgentType })}
              className="w-full px-3 py-2 border rounded text-sm"
            >
              {Object.entries(AGENT_TYPE_LABELS).map(([value, label]) => (
                <option key={value} value={value}>{label}</option>
              ))}
            </select>
          </div>
          <div className="flex gap-2">
            <button
              onClick={handleCreate}
              className="flex-1 px-4 py-2 bg-blue-500 text-white rounded text-sm hover:bg-blue-600"
            >
              Create Agent
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

      {/* Agent List */}
      {agents.length === 0 ? (
        <div className="p-8 text-center text-gray-400">
          No sub-agents created yet. Click "Create" to add one.
        </div>
      ) : (
        <div className="space-y-2">
          {agents.map((agent) => (
            <div
              key={agent.id}
              className="p-4 border rounded-lg hover:bg-gray-50"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div>
                    <h3 className="font-medium">{agent.name}</h3>
                    <div className="flex items-center gap-2 text-sm text-gray-500">
                      <span>{AGENT_TYPE_LABELS[agent.type]}</span>
                      {getStatusBadge(agent.status)}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {agent.status === 'idle' && (
                    <button
                      onClick={() => {/* Open task assignment modal */}}
                      className="p-2 hover:bg-blue-100 rounded text-blue-500"
                      title="Assign task"
                    >
                      <Play className="w-4 h-4" />
                    </button>
                  )}
                  {agent.status === 'running' && (
                    <button
                      onClick={() => {/* Pause agent */}}
                      className="p-2 hover:bg-yellow-100 rounded text-yellow-500"
                      title="Pause"
                    >
                      <Pause className="w-4 h-4" />
                    </button>
                  )}
                  <button
                    onClick={() => deleteAgent(agent.id)}
                    className="p-2 hover:bg-red-100 rounded text-red-500"
                    title="Delete"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
              {agent.task && (
                <div className="mt-2 text-sm text-gray-600">
                  <span className="font-medium">Task:</span> {agent.task}
                </div>
              )}
              {agent.error && (
                <div className="mt-2 text-sm text-red-500">
                  <span className="font-medium">Error:</span> {agent.error}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default SubAgentList;
