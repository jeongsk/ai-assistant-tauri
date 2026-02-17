/**
 * File Explorer Component
 */

import React, { useState, useEffect } from 'react';
import { Folder, File, ChevronRight, ChevronDown, RefreshCw, Home } from 'lucide-react';
import { useFiles, FileEntry } from '../../hooks/useFiles';
import { useSettingsStore } from '../../stores/settingsStore';

export function FileExplorer() {
  const { currentPath, files, loading, error, navigateTo, readFile } = useFiles();
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [fileContent, setFileContent] = useState<string | null>(null);
  
  const folderPermissions = useSettingsStore((state) => state.folderPermissions);

  // Navigate to first allowed folder on mount
  useEffect(() => {
    if (!currentPath && folderPermissions.length > 0) {
      navigateTo(folderPermissions[0].path);
    }
  }, [currentPath, folderPermissions, navigateTo]);

  const toggleDir = (path: string) => {
    const newExpanded = new Set(expandedDirs);
    if (newExpanded.has(path)) {
      newExpanded.delete(path);
    } else {
      newExpanded.add(path);
    }
    setExpandedDirs(newExpanded);
  };

  const handleFileClick = async (path: string) => {
    setSelectedFile(path);
    try {
      const content = await readFile(path);
      setFileContent(content);
    } catch (err) {
      setFileContent(`Error: ${err}`);
    }
  };

  const handleRefresh = () => {
    if (currentPath) {
      navigateTo(currentPath);
    }
  };

  if (folderPermissions.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500">
        <div className="text-center">
          <Folder className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <h2 className="text-lg font-medium">No Folders Added</h2>
          <p className="text-sm">Add folders in Settings to browse files</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      {/* File Tree */}
      <div className="w-64 border-r flex flex-col bg-white dark:bg-gray-900">
        {/* Header */}
        <div className="p-3 border-b flex items-center justify-between">
          <span className="text-sm font-medium">Files</span>
          <button
            onClick={handleRefresh}
            className="p-1 hover:bg-gray-100 rounded"
            title="Refresh"
          >
            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {/* Folder Selector */}
        <div className="p-2 border-b">
          <select
            value={currentPath}
            onChange={(e) => navigateTo(e.target.value)}
            className="w-full px-2 py-1.5 text-sm border rounded bg-white dark:bg-gray-800"
          >
            {folderPermissions.map((perm) => (
              <option key={perm.id} value={perm.path}>
                {perm.path.split('/').pop() || perm.path}
              </option>
            ))}
          </select>
        </div>

        {/* File List */}
        <div className="flex-1 overflow-y-auto p-2">
          {error && (
            <p className="text-sm text-red-500 p-2">{error}</p>
          )}
          
          {files.length === 0 && !loading && !error && (
            <p className="text-sm text-gray-400 p-2">Empty folder</p>
          )}

          <div className="space-y-0.5">
            {files.map((entry) => (
              <FileTreeItem
                key={entry.name}
                entry={entry}
                currentPath={currentPath}
                expanded={expandedDirs.has(`${currentPath}/${entry.name}`)}
                selected={selectedFile === `${currentPath}/${entry.name}`}
                onToggle={() => toggleDir(`${currentPath}/${entry.name}`)}
                onClick={() => {
                  if (!entry.isDir) {
                    handleFileClick(`${currentPath}/${entry.name}`);
                  }
                }}
              />
            ))}
          </div>
        </div>
      </div>

      {/* File Preview */}
      <div className="flex-1 flex flex-col">
        {selectedFile ? (
          <>
            <div className="p-3 border-b bg-gray-50 dark:bg-gray-800">
              <p className="text-sm font-mono">{selectedFile}</p>
            </div>
            <div className="flex-1 overflow-auto p-4">
              <pre className="text-sm font-mono whitespace-pre-wrap">
                {fileContent || 'Loading...'}
              </pre>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500">
            <div className="text-center">
              <File className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p className="text-sm">Select a file to preview</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

interface FileTreeItemProps {
  entry: FileEntry;
  currentPath: string;
  expanded: boolean;
  selected: boolean;
  onToggle: () => void;
  onClick: () => void;
}

function FileTreeItem({ entry, expanded, selected, onToggle, onClick }: FileTreeItemProps) {
  const Icon = entry.isDir ? Folder : File;
  
  return (
    <div
      onClick={entry.isDir ? onToggle : onClick}
      className={`flex items-center gap-1.5 px-2 py-1.5 rounded cursor-pointer text-sm ${
        selected ? 'bg-blue-100 dark:bg-blue-900/30' : 'hover:bg-gray-100 dark:hover:bg-gray-800'
      }`}
    >
      {entry.isDir && (
        <span className="w-4 h-4 flex items-center justify-center">
          {expanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
        </span>
      )}
      {!entry.isDir && <span className="w-4" />}
      <Icon className={`w-4 h-4 ${entry.isDir ? 'text-yellow-500' : 'text-gray-400'}`} />
      <span className="truncate">{entry.name}</span>
    </div>
  );
}
