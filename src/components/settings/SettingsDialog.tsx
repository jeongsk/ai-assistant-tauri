/**
 * Settings Dialog Component
 */

import React, { useState } from "react";
import {
  Settings,
  X,
  Plus,
  Trash2,
  Eye,
  EyeOff,
  Check,
  AlertCircle,
} from "lucide-react";
import {
  useSettingsStore,
  ProviderConfig,
  FolderPermission,
} from "../../stores/settingsStore";
import { validateFolderPath } from "../../services/tauri";

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsDialog({ isOpen, onClose }: SettingsDialogProps) {
  const [activeTab, setActiveTab] = useState<
    "providers" | "folders" | "appearance"
  >("providers");

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-900 rounded-lg shadow-xl w-full max-w-2xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <div className="flex items-center gap-2">
            <Settings className="w-5 h-5" />
            <h2 className="text-lg font-semibold">Settings</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b">
          <button
            onClick={() => setActiveTab("providers")}
            className={`px-4 py-2 text-sm ${activeTab === "providers" ? "border-b-2 border-blue-500 text-blue-500" : "text-gray-600"}`}
          >
            Providers
          </button>
          <button
            onClick={() => setActiveTab("folders")}
            className={`px-4 py-2 text-sm ${activeTab === "folders" ? "border-b-2 border-blue-500 text-blue-500" : "text-gray-600"}`}
          >
            Folders
          </button>
          <button
            onClick={() => setActiveTab("appearance")}
            className={`px-4 py-2 text-sm ${activeTab === "appearance" ? "border-b-2 border-blue-500 text-blue-500" : "text-gray-600"}`}
          >
            Appearance
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {activeTab === "providers" && <ProviderSettings />}
          {activeTab === "folders" && <FolderSettings />}
          {activeTab === "appearance" && <AppearanceSettings />}
        </div>
      </div>
    </div>
  );
}

function ProviderSettings() {
  const providers = useSettingsStore((state) => state.providers);
  const activeProvider = useSettingsStore((state) => state.activeProvider);
  const setProvider = useSettingsStore((state) => state.setProvider);
  const setActiveProvider = useSettingsStore(
    (state) => state.setActiveProvider,
  );
  const syncProvidersToAgent = useSettingsStore(
    (state) => state.syncProvidersToAgent,
  );

  const handleProviderUpdate = (
    id: string,
    updates: Partial<ProviderConfig>,
  ) => {
    setProvider(id, { ...providers[id], ...updates });
    // Sync to agent runtime after update
    syncProvidersToAgent();
  };

  const handleProviderSelect = (id: string) => {
    setActiveProvider(id);
    // Sync to agent runtime after selection
    syncProvidersToAgent();
  };

  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500 mb-4">
        Configure your AI providers. You can switch between them anytime.
      </p>

      {Object.entries(providers).map(([id, config]) => (
        <ProviderCard
          key={id}
          id={id}
          config={config}
          isActive={id === activeProvider}
          onSelect={() => handleProviderSelect(id)}
          onUpdate={(updates) => handleProviderUpdate(id, updates)}
        />
      ))}
    </div>
  );
}

interface ProviderCardProps {
  id: string;
  config: ProviderConfig;
  isActive: boolean;
  onSelect: () => void;
  onUpdate: (updates: Partial<ProviderConfig>) => void;
}

function ProviderCard({
  id,
  config,
  isActive,
  onSelect,
  onUpdate,
}: ProviderCardProps) {
  const [showKey, setShowKey] = useState(false);

  return (
    <div
      className={`p-4 border rounded-lg ${isActive ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <input
            type="radio"
            checked={isActive}
            onChange={onSelect}
            className="w-4 h-4"
          />
          <span className="font-medium capitalize">{id}</span>
          {config.type === "ollama" && (
            <span className="text-xs px-2 py-0.5 bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 rounded">
              Local
            </span>
          )}
        </div>
        {isActive && <Check className="w-4 h-4 text-blue-500" />}
      </div>

      {"apiKey" in config && config.type !== "ollama" && (
        <div className="space-y-2">
          <label className="block text-sm text-gray-600 dark:text-gray-400">
            API Key
          </label>
          <div className="flex gap-2">
            <input
              type={showKey ? "text" : "password"}
              value={config.apiKey || ""}
              onChange={(e) => onUpdate({ apiKey: e.target.value })}
              placeholder={`Enter your ${id} API key`}
              className="flex-1 px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
            />
            <button
              onClick={() => setShowKey(!showKey)}
              className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
            >
              {showKey ? (
                <EyeOff className="w-4 h-4" />
              ) : (
                <Eye className="w-4 h-4" />
              )}
            </button>
          </div>
        </div>
      )}

      {config.type === "ollama" && (
        <div className="space-y-2">
          <label className="block text-sm text-gray-600 dark:text-gray-400">
            Base URL
          </label>
          <input
            type="text"
            value={config.baseUrl || ""}
            onChange={(e) => onUpdate({ baseUrl: e.target.value })}
            placeholder="http://localhost:11434"
            className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
          />
        </div>
      )}

      <div className="space-y-2 mt-3">
        <label className="block text-sm text-gray-600 dark:text-gray-400">
          Model
        </label>
        <input
          type="text"
          value={config.model}
          onChange={(e) => onUpdate({ model: e.target.value })}
          className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
        />
      </div>
    </div>
  );
}

function FolderSettings() {
  const folderPermissions = useSettingsStore(
    (state) => state.folderPermissions,
  );
  const addFolderPermission = useSettingsStore(
    (state) => state.addFolderPermission,
  );
  const removeFolderPermission = useSettingsStore(
    (state) => state.removeFolderPermission,
  );

  const [newPath, setNewPath] = useState("");
  const [newLevel, setNewLevel] = useState<"read" | "readwrite">("readwrite");
  const [error, setError] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);

  const handleAddFolder = async () => {
    if (!newPath.trim()) return;

    setAdding(true);
    setError(null);

    try {
      // Validate path
      await validateFolderPath(newPath.trim());

      // Check if already added
      if (folderPermissions.some((p) => p.path === newPath.trim())) {
        setError("This folder is already added");
        return;
      }

      addFolderPermission({
        id: crypto.randomUUID(),
        path: newPath.trim(),
        level: newLevel,
      });

      setNewPath("");
      setNewLevel("readwrite");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Invalid path");
    } finally {
      setAdding(false);
    }
  };

  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500 mb-4">
        Grant the AI access to specific folders on your computer.
      </p>

      {/* Add folder form */}
      <div className="p-3 border rounded-lg bg-gray-50 dark:bg-gray-800/50 space-y-3">
        <div>
          <label className="block text-sm font-medium mb-1">Folder Path</label>
          <input
            type="text"
            value={newPath}
            onChange={(e) => setNewPath(e.target.value)}
            placeholder="/Users/username/Documents"
            className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">
            Permission Level
          </label>
          <select
            value={newLevel}
            onChange={(e) =>
              setNewLevel(e.target.value as "read" | "readwrite")
            }
            className="w-full px-3 py-2 border rounded text-sm bg-white dark:bg-gray-800"
          >
            <option value="read">Read only</option>
            <option value="readwrite">Read & Write</option>
          </select>
        </div>

        {error && (
          <div className="flex items-center gap-2 text-sm text-red-500">
            <AlertCircle className="w-4 h-4" />
            {error}
          </div>
        )}

        <button
          onClick={handleAddFolder}
          disabled={!newPath.trim() || adding}
          className="flex items-center justify-center gap-2 w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Plus className="w-4 h-4" />
          Add Folder
        </button>
      </div>

      {/* Folder list */}
      {folderPermissions.length === 0 ? (
        <p className="text-sm text-gray-400 text-center py-4">
          No folders added yet.
        </p>
      ) : (
        <div className="space-y-2">
          <h3 className="text-sm font-medium text-gray-600 dark:text-gray-400">
            Allowed Folders
          </h3>
          {folderPermissions.map((permission) => (
            <div
              key={permission.id}
              className="flex items-center justify-between p-3 border rounded-lg"
            >
              <div className="flex-1 min-w-0">
                <p className="font-mono text-sm truncate">{permission.path}</p>
                <span
                  className={`text-xs ${
                    permission.level === "read"
                      ? "text-yellow-600"
                      : "text-green-600"
                  }`}
                >
                  {permission.level === "read"
                    ? "🔒 Read only"
                    : "✏️ Read & Write"}
                </span>
              </div>
              <button
                onClick={() => removeFolderPermission(permission.id)}
                className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function AppearanceSettings() {
  const theme = useSettingsStore((state) => state.theme);
  const setTheme = useSettingsStore((state) => state.setTheme);

  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-2">Theme</label>
        <div className="flex gap-2">
          {(["light", "dark", "system"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTheme(t)}
              className={`px-4 py-2 rounded capitalize text-sm ${
                theme === t
                  ? "bg-blue-500 text-white"
                  : "bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700"
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
