/**
 * SkillList Component - Display and manage skills
 */

import { useState, useEffect } from 'react';
import { Plus, Search, Code, Trash2, Edit, Play } from 'lucide-react';
import { useSkillStore } from '../../stores/skillStore';
import type { Skill } from '../../types/skill';

interface SkillListProps {
  onEditSkill: (skill: Skill) => void;
  onInvokeSkill: (skill: Skill) => void;
}

export function SkillList({ onEditSkill, onInvokeSkill }: SkillListProps) {
  const { skills, loading, loadSkills, deleteSkill } = useSkillStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [showCreate, setShowCreate] = useState(false);

  useEffect(() => {
    loadSkills();
  }, [loadSkills]);

  const filteredSkills = skills.filter(
    (skill) =>
      skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this skill?')) {
      await deleteSkill(id);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Skills</h2>
          <button
            onClick={() => setShowCreate(true)}
            className="flex items-center gap-2 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          >
            <Plus className="w-4 h-4" />
            New Skill
          </button>
        </div>

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search skills..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Skill Grid */}
      <div className="flex-1 overflow-auto p-4">
        {filteredSkills.length === 0 ? (
          <div className="text-center py-12 text-gray-500">
            <Code className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>No skills found</p>
            <p className="text-sm mt-2">Create a skill to get started</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredSkills.map((skill) => (
              <div
                key={skill.id}
                className="border rounded-lg p-4 hover:shadow-md transition-shadow"
              >
                <div className="flex items-start justify-between mb-2">
                  <h3 className="font-medium">{skill.name}</h3>
                  <div className="flex gap-1">
                    <button
                      onClick={() => onInvokeSkill(skill)}
                      className="p-1 hover:bg-green-100 rounded text-green-600"
                      title="Invoke skill"
                    >
                      <Play className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => onEditSkill(skill)}
                      className="p-1 hover:bg-gray-100 rounded"
                      title="Edit skill"
                    >
                      <Edit className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handleDelete(skill.id)}
                      className="p-1 hover:bg-red-100 rounded text-red-600"
                      title="Delete skill"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <p className="text-sm text-gray-600 line-clamp-2 mb-3">
                  {skill.description}
                </p>

                {skill.tools.length > 0 && (
                  <div className="flex flex-wrap gap-1">
                    {skill.tools.slice(0, 3).map((tool) => (
                      <span
                        key={tool}
                        className="text-xs px-2 py-0.5 bg-gray-100 rounded-full"
                      >
                        {tool}
                      </span>
                    ))}
                    {skill.tools.length > 3 && (
                      <span className="text-xs px-2 py-0.5 bg-gray-100 rounded-full">
                        +{skill.tools.length - 3}
                      </span>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Modal - Placeholder */}
      {showCreate && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-lg w-full mx-4">
            <h3 className="text-lg font-semibold mb-4">Create New Skill</h3>
            <p className="text-gray-600 dark:text-gray-400">
              Skill editor will be implemented here.
            </p>
            <div className="flex justify-end mt-4">
              <button
                onClick={() => setShowCreate(false)}
                className="px-4 py-2 border rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
