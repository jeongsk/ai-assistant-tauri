/**
 * Memory Panel Component - View and manage memories
 */

import React, { useState } from "react";
import { Brain, Search, Trash2, Filter, Clock } from "lucide-react";
import { useMemoryStore } from "../../stores/memoryStore";
import { MemoryType } from "../../types/memory";

export function MemoryPanel() {
  const { memories, patterns, isLoading, deleteMemory, searchMemories } = useMemoryStore();
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState<MemoryType | "all">("all");

  const filteredMemories = memories.filter((m) => {
    if (filterType !== "all" && m.type !== filterType) return false;
    if (searchQuery && !m.content.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const getTypeLabel = (type: MemoryType) => {
    switch (type) {
      case "episodic": return "Event";
      case "semantic": return "Fact";
      case "procedural": return "Workflow";
    }
  };

  const getTypeColor = (type: MemoryType) => {
    switch (type) {
      case "episodic": return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
      case "semantic": return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400";
      case "procedural": return "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400";
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 mb-4">
        <Brain className="w-5 h-5" />
        <h3 className="text-lg font-semibold">Memory</h3>
      </div>

      {/* Search and Filter */}
      <div className="flex gap-2">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search memories..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
          />
        </div>
        <select
          value={filterType}
          onChange={(e) => setFilterType(e.target.value as MemoryType | "all")}
          className="px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        >
          <option value="all">All Types</option>
          <option value="episodic">Events</option>
          <option value="semantic">Facts</option>
          <option value="procedural">Workflows</option>
        </select>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-2">
        <div className="p-3 border rounded text-center">
          <div className="text-2xl font-bold">{memories.length}</div>
          <div className="text-xs text-gray-500">Total</div>
        </div>
        <div className="p-3 border rounded text-center">
          <div className="text-2xl font-bold">{patterns.length}</div>
          <div className="text-xs text-gray-500">Patterns</div>
        </div>
        <div className="p-3 border rounded text-center">
          <div className="text-2xl font-bold">{memories.filter(m => m.importance > 0.7).length}</div>
          <div className="text-xs text-gray-500">Important</div>
        </div>
      </div>

      {/* Memory List */}
      {isLoading ? (
        <div className="text-center py-8 text-gray-500">Loading...</div>
      ) : filteredMemories.length === 0 ? (
        <div className="text-center py-8 text-gray-400">
          No memories found
        </div>
      ) : (
        <div className="space-y-2 max-h-96 overflow-y-auto">
          {filteredMemories.map((memory) => (
            <div
              key={memory.id}
              className="p-3 border rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800"
            >
              <div className="flex items-start justify-between gap-2">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className={`px-2 py-0.5 text-xs rounded ${getTypeColor(memory.type)}`}>
                      {getTypeLabel(memory.type)}
                    </span>
                    <span className="text-xs text-gray-400">
                      {Math.round(memory.importance * 100)}% important
                    </span>
                  </div>
                  <p className="text-sm truncate">{memory.content}</p>
                  <div className="flex items-center gap-1 mt-1 text-xs text-gray-400">
                    <Clock className="w-3 h-3" />
                    {new Date(memory.createdAt).toLocaleDateString()}
                  </div>
                </div>
                <button
                  onClick={() => deleteMemory(memory.id)}
                  className="p-1 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
