/**
 * Template Library Component - Browse and manage templates
 */

import React, { useState } from "react";
import { FileText, Plus, Trash2, Copy, Search, Filter } from "lucide-react";
import { useCollaborationStore } from "../../stores/collaborationStore";
import { Template, Visibility } from "../../types/collaboration";

export function TemplateLibrary() {
  const { templates, createTemplate, deleteTemplate } = useCollaborationStore();
  const [searchQuery, setSearchQuery] = useState("");
  const [filterVisibility, setFilterVisibility] = useState<Visibility | "all">("all");

  const filteredTemplates = templates.filter((t) => {
    if (filterVisibility !== "all" && t.visibility !== filterVisibility) return false;
    if (searchQuery && !t.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  const categories = [...new Set(templates.map((t) => t.category))];

  const getVisibilityColor = (visibility: Visibility) => {
    switch (visibility) {
      case "public":
        return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400";
      case "team":
        return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
      case "private":
        return "bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300";
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <FileText className="w-5 h-5" />
          <h3 className="text-lg font-semibold">Templates</h3>
        </div>
        <button
          onClick={() => {
            createTemplate({
              name: "New Template",
              category: "general",
              content: "",
              visibility: "private",
              version: "1.0.0",
            });
          }}
          className="flex items-center gap-1 px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          <Plus className="w-4 h-4" />
          New
        </button>
      </div>

      {/* Search and Filter */}
      <div className="flex gap-2">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search templates..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
          />
        </div>
        <select
          value={filterVisibility}
          onChange={(e) => setFilterVisibility(e.target.value as Visibility | "all")}
          className="px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        >
          <option value="all">All</option>
          <option value="private">Private</option>
          <option value="public">Public</option>
          <option value="team">Team</option>
        </select>
      </div>

      {/* Template List */}
      {filteredTemplates.length === 0 ? (
        <div className="text-center py-8 text-gray-400">
          No templates found
        </div>
      ) : (
        <div className="space-y-2">
          {filteredTemplates.map((template) => (
            <div
              key={template.id}
              className="p-4 border rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <h4 className="font-medium">{template.name}</h4>
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${getVisibilityColor(
                        template.visibility
                      )}`}
                    >
                      {template.visibility}
                    </span>
                    <span className="text-xs text-gray-400">
                      {template.category}
                    </span>
                  </div>
                  <p className="text-sm text-gray-500 line-clamp-2">
                    {template.content.slice(0, 100)}...
                  </p>
                </div>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(template.content);
                    }}
                    className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
                    title="Copy"
                  >
                    <Copy className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => deleteTemplate(template.id)}
                    className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
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
