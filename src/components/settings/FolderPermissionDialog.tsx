// FolderPermissionDialog Component

import React, { useState } from 'react';
import { Folder, Plus, Trash2, Eye, Edit3 } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { usePermissionStore } from '../../stores/permissionStore';
import type { PermissionLevel } from '../../types/permission';

interface FolderPermissionDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export const FolderPermissionDialog: React.FC<FolderPermissionDialogProps> = ({
  isOpen,
  onClose,
}) => {
  const [selectedPath, setSelectedPath] = useState<string>('');
  const [selectedLevel, setSelectedLevel] = useState<PermissionLevel>('read');
  const [error, setError] = useState<string | null>(null);

  const {
    permissions,
    addPermission,
    removePermission,
    updatePermission,
    isLoading,
  } = usePermissionStore();

  const handleBrowseFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Folder',
      });

      if (selected && typeof selected === 'string') {
        setSelectedPath(selected);
        setError(null);
      }
    } catch (err) {
      console.error('Failed to browse folder:', err);
      setError('Failed to open folder dialog');
    }
  };

  const handleAddPermission = async () => {
    if (!selectedPath) {
      setError('Please select a folder');
      return;
    }

    // Check if already exists
    if (permissions.some((p) => p.path === selectedPath)) {
      setError('This folder is already added');
      return;
    }

    try {
      await addPermission(selectedPath, selectedLevel);
      setSelectedPath('');
      setSelectedLevel('read');
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleRemovePermission = async (id: string) => {
    try {
      await removePermission(id);
    } catch (err) {
      console.error('Failed to remove permission:', err);
    }
  };

  const handleUpdateLevel = async (id: string, level: PermissionLevel) => {
    try {
      await updatePermission(id, level);
    } catch (err) {
      console.error('Failed to update permission:', err);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-lg mx-4">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b dark:border-gray-700">
          <h2 className="text-lg font-semibold">Folder Permissions</h2>
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
          >
            ×
          </button>
        </div>

        {/* Content */}
        <div className="p-4">
          {/* Add new permission */}
          <div className="mb-6">
            <label className="block text-sm font-medium mb-2">Add Folder</label>
            <div className="flex gap-2">
              <button
                onClick={handleBrowseFolder}
                className="flex items-center gap-2 px-3 py-2 border rounded-lg
                         hover:bg-gray-100 dark:hover:bg-gray-700
                         dark:border-gray-600"
              >
                <Folder className="w-4 h-4" />
                Browse
              </button>
              <input
                type="text"
                value={selectedPath}
                onChange={(e) => setSelectedPath(e.target.value)}
                placeholder="Selected folder path"
                className="flex-1 px-3 py-2 border rounded-lg
                         dark:bg-gray-700 dark:border-gray-600"
                readOnly
              />
            </div>

            {selectedPath && (
              <div className="mt-3">
                <label className="block text-sm font-medium mb-2">
                  Permission Level
                </label>
                <div className="flex gap-4">
                  <label className="flex items-center gap-2">
                    <input
                      type="radio"
                      name="level"
                      value="read"
                      checked={selectedLevel === 'read'}
                      onChange={() => setSelectedLevel('read')}
                    />
                    <Eye className="w-4 h-4" />
                    Read Only
                  </label>
                  <label className="flex items-center gap-2">
                    <input
                      type="radio"
                      name="level"
                      value="readwrite"
                      checked={selectedLevel === 'readwrite'}
                      onChange={() => setSelectedLevel('readwrite')}
                    />
                    <Edit3 className="w-4 h-4" />
                    Read & Write
                  </label>
                </div>
              </div>
            )}

            {error && (
              <p className="mt-2 text-sm text-red-500">{error}</p>
            )}

            {selectedPath && (
              <button
                onClick={handleAddPermission}
                disabled={isLoading}
                className="mt-3 flex items-center gap-2 px-4 py-2
                         bg-blue-500 text-white rounded-lg hover:bg-blue-600
                         disabled:opacity-50"
              >
                <Plus className="w-4 h-4" />
                Add Permission
              </button>
            )}
          </div>

          {/* Existing permissions */}
          <div>
            <h3 className="text-sm font-medium mb-2">Current Permissions</h3>
            {permissions.length === 0 ? (
              <p className="text-sm text-gray-500 dark:text-gray-400">
                No folder permissions configured
              </p>
            ) : (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {permissions.map((perm) => (
                  <div
                    key={perm.id}
                    className="flex items-center gap-2 p-2 border rounded-lg
                             dark:border-gray-600"
                  >
                    <Folder className="w-4 h-4 text-yellow-500" />
                    <span className="flex-1 text-sm truncate">{perm.path}</span>

                    <select
                      value={perm.level}
                      onChange={(e) =>
                        handleUpdateLevel(perm.id, e.target.value as PermissionLevel)
                      }
                      className="px-2 py-1 text-sm border rounded
                               dark:bg-gray-700 dark:border-gray-600"
                    >
                      <option value="read">Read</option>
                      <option value="readwrite">Read & Write</option>
                    </select>

                    <button
                      onClick={() => handleRemovePermission(perm.id)}
                      className="p-1 text-red-500 hover:bg-red-100 rounded
                               dark:hover:bg-red-900"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end px-4 py-3 border-t dark:border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded-lg
                     hover:bg-gray-200 dark:hover:bg-gray-600"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default FolderPermissionDialog;
