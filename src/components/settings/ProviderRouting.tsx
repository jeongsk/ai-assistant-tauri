/**
 * Provider Routing Settings Component
 */

import React, { useState, useEffect } from 'react';
import { Plus, Trash2, ChevronUp, ChevronDown, ToggleLeft, ToggleRight } from 'lucide-react';
import { useRouterStore } from '../../stores/routerStore';
import type { RoutingRule, TaskType } from '../../stores/routerStore';

const TASK_TYPE_LABELS: Record<TaskType, string> = {
  coding: 'Coding',
  creative: 'Creative',
  analysis: 'Analysis',
  chat: 'Chat',
  research: 'Research',
  planning: 'Planning',
};

export function ProviderRouting() {
  const { rules, fallbackChain, loading, error, loadRules, toggleRule, removeRule } = useRouterStore();
  const [showAdd, setShowAdd] = useState(false);

  useEffect(() => {
    loadRules();
  }, [loadRules]);

  if (loading) {
    return <div className="p-4 text-center text-gray-500">Loading...</div>;
  }

  return (
    <div className="space-y-4">
      {error && (
        <div className="p-3 bg-red-50 text-red-600 rounded text-sm">{error}</div>
      )}

      <p className="text-sm text-gray-500">
        Configure how tasks are routed to different AI providers based on task type and complexity.
      </p>

      {/* Fallback Chain */}
      <div className="p-4 border rounded-lg">
        <h3 className="font-medium mb-2">Fallback Chain</h3>
        <p className="text-sm text-gray-500 mb-3">
          Providers are tried in order if the primary fails.
        </p>
        <div className="flex gap-2">
          {fallbackChain.map((provider, index) => (
            <div key={provider} className="flex items-center gap-1">
              <span className="px-3 py-1 bg-blue-100 text-blue-700 rounded text-sm">
                {provider}
              </span>
              {index < fallbackChain.length - 1 && (
                <span className="text-gray-400">→</span>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Routing Rules */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="font-medium">Routing Rules</h3>
          <button
            onClick={() => setShowAdd(!showAdd)}
            className="flex items-center gap-1 px-3 py-1.5 bg-blue-500 text-white rounded text-sm hover:bg-blue-600"
          >
            <Plus className="w-4 h-4" />
            Add Rule
          </button>
        </div>

        {rules.length === 0 ? (
          <div className="p-4 text-center text-gray-400 border rounded">
            No routing rules configured.
          </div>
        ) : (
          <div className="space-y-2">
            {rules.map((rule) => (
              <RuleCard
                key={rule.id}
                rule={rule}
                onToggle={() => toggleRule(rule.id)}
                onDelete={() => removeRule(rule.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface RuleCardProps {
  rule: RoutingRule;
  onToggle: () => void;
  onDelete: () => void;
}

function RuleCard({ rule, onToggle, onDelete }: RuleCardProps) {
  return (
    <div className={`p-3 border rounded-lg ${!rule.enabled ? 'opacity-50' : ''}`}>
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <div className="flex items-center gap-2">
            <h4 className="font-medium">{rule.name}</h4>
            <span className="text-xs text-gray-500">Priority: {rule.priority}</span>
          </div>
          <div className="flex items-center gap-2 mt-1 text-sm text-gray-500">
            <span className="px-2 py-0.5 bg-gray-100 rounded">
              {rule.provider}
            </span>
            {rule.model && (
              <span className="text-xs">{rule.model}</span>
            )}
          </div>
          {rule.condition.taskTypes && rule.condition.taskTypes.length > 0 && (
            <div className="flex gap-1 mt-2">
              {rule.condition.taskTypes.map((type) => (
                <span
                  key={type}
                  className="px-2 py-0.5 bg-blue-50 text-blue-600 text-xs rounded"
                >
                  {TASK_TYPE_LABELS[type]}
                </span>
              ))}
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onToggle}
            className={`p-2 rounded ${rule.enabled ? 'text-blue-500' : 'text-gray-400'}`}
          >
            {rule.enabled ? (
              <ToggleRight className="w-5 h-5" />
            ) : (
              <ToggleLeft className="w-5 h-5" />
            )}
          </button>
          <button
            onClick={onDelete}
            className="p-2 hover:bg-red-100 rounded text-red-500"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}

export default ProviderRouting;
