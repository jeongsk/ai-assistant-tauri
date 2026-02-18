/**
 * Integrations Panel Component - Manage external integrations
 */

import React, { useState } from "react";
import { Database, GitBranch, Cloud, Plus, Trash2, Check, X } from "lucide-react";
import { useIntegrationsStore } from "../../stores/integrationsStore";
import { DatabaseConfig, GitConfig, CloudStorageConfig } from "../../types/integration";

type TabType = "database" | "git" | "cloud";

export function IntegrationsPanel() {
  const [activeTab, setActiveTab] = useState<TabType>("database");
  const { databases, gitRepos, cloudStorages, addDatabase, addGitRepo, addCloudStorage, removeDatabase, removeGitRepo, removeCloudStorage } = useIntegrationsStore();

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold">Integrations</h3>

      {/* Tabs */}
      <div className="flex border-b">
        <button
          onClick={() => setActiveTab("database")}
          className={`flex items-center gap-1 px-4 py-2 text-sm ${
            activeTab === "database" ? "border-b-2 border-blue-500 text-blue-500" : "text-gray-600"
          }`}
        >
          <Database className="w-4 h-4" />
          Database
        </button>
        <button
          onClick={() => setActiveTab("git")}
          className={`flex items-center gap-1 px-4 py-2 text-sm ${
            activeTab === "git" ? "border-b-2 border-blue-500 text-blue-500" : "text-gray-600"
          }`}
        >
          <GitBranch className="w-4 h-4" />
          Git
        </button>
        <button
          onClick={() => setActiveTab("cloud")}
          className={`flex items-center gap-1 px-4 py-2 text-sm ${
            activeTab === "cloud" ? "border-b-2 border-blue-500 text-blue-500" : "text-gray-600"
          }`}
        >
          <Cloud className="w-4 h-4" />
          Cloud
        </button>
      </div>

      {/* Content */}
      {activeTab === "database" && (
        <DatabaseSection
          databases={databases}
          onAdd={addDatabase}
          onRemove={removeDatabase}
        />
      )}
      {activeTab === "git" && (
        <GitSection
          repos={gitRepos}
          onAdd={addGitRepo}
          onRemove={removeGitRepo}
        />
      )}
      {activeTab === "cloud" && (
        <CloudSection
          storages={cloudStorages}
          onAdd={addCloudStorage}
          onRemove={removeCloudStorage}
        />
      )}
    </div>
  );
}

function DatabaseSection({
  databases,
  onAdd,
  onRemove,
}: {
  databases: DatabaseConfig[];
  onAdd: (config: DatabaseConfig) => void;
  onRemove: (name: string) => void;
}) {
  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500">
        Connect to PostgreSQL or MySQL databases.
      </p>

      {databases.length === 0 ? (
        <div className="text-center py-6 text-gray-400">
          No databases connected
        </div>
      ) : (
        <div className="space-y-2">
          {databases.map((db) => (
            <div key={db.database} className="p-3 border rounded-lg flex justify-between items-center">
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-medium">{db.database}</span>
                  <span className="text-xs px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded uppercase">
                    {db.type}
                  </span>
                </div>
                <p className="text-sm text-gray-500">{db.host}:{db.port}</p>
              </div>
              <button
                onClick={() => onRemove(db.database)}
                className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      )}

      <button className="flex items-center gap-2 w-full px-4 py-2 border rounded hover:bg-gray-50 dark:hover:bg-gray-800">
        <Plus className="w-4 h-4" />
        Add Database
      </button>
    </div>
  );
}

function GitSection({
  repos,
  onAdd,
  onRemove,
}: {
  repos: GitConfig[];
  onAdd: (config: GitConfig) => void;
  onRemove: (path: string) => void;
}) {
  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500">
        Manage Git repositories.
      </p>

      {repos.length === 0 ? (
        <div className="text-center py-6 text-gray-400">
          No repositories configured
        </div>
      ) : (
        <div className="space-y-2">
          {repos.map((repo) => (
            <div key={repo.repositoryPath} className="p-3 border rounded-lg flex justify-between items-center">
              <div>
                <p className="font-medium font-mono text-sm">{repo.repositoryPath}</p>
                {repo.userName && (
                  <p className="text-sm text-gray-500">{repo.userName} &lt;{repo.userEmail}&gt;</p>
                )}
              </div>
              <button
                onClick={() => onRemove(repo.repositoryPath)}
                className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      )}

      <button className="flex items-center gap-2 w-full px-4 py-2 border rounded hover:bg-gray-50 dark:hover:bg-gray-800">
        <Plus className="w-4 h-4" />
        Add Repository
      </button>
    </div>
  );
}

function CloudSection({
  storages,
  onAdd,
  onRemove,
}: {
  storages: CloudStorageConfig[];
  onAdd: (config: CloudStorageConfig) => void;
  onRemove: (bucket: string) => void;
}) {
  return (
    <div className="space-y-4">
      <p className="text-sm text-gray-500">
        Connect to AWS S3 or Google Cloud Storage.
      </p>

      {storages.length === 0 ? (
        <div className="text-center py-6 text-gray-400">
          No cloud storage connected
        </div>
      ) : (
        <div className="space-y-2">
          {storages.map((storage) => (
            <div key={storage.bucket} className="p-3 border rounded-lg flex justify-between items-center">
              <div>
                <div className="flex items-center gap-2">
                  <span className="font-medium">{storage.bucket}</span>
                  <span className="text-xs px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded uppercase">
                    {storage.provider}
                  </span>
                </div>
                {storage.region && (
                  <p className="text-sm text-gray-500">{storage.region}</p>
                )}
              </div>
              <button
                onClick={() => onRemove(storage.bucket)}
                className="p-2 hover:bg-red-100 dark:hover:bg-red-900/30 rounded text-red-500"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      )}

      <button className="flex items-center gap-2 w-full px-4 py-2 border rounded hover:bg-gray-50 dark:hover:bg-gray-800">
        <Plus className="w-4 h-4" />
        Add Cloud Storage
      </button>
    </div>
  );
}
