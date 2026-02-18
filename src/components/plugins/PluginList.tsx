/**
 * Plugin List Component - Manage installed plugins
 */

import React from "react";
import { Plug, Trash2, Power, PowerOff, Package } from "lucide-react";
import { usePluginStore } from "../../stores/pluginStore";

export function PluginList() {
  const { plugins, isLoading, enablePlugin, disablePlugin, uninstallPlugin } = usePluginStore();

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 mb-4">
        <Plug className="w-5 h-5" />
        <h3 className="text-lg font-semibold">Plugins</h3>
      </div>

      {isLoading ? (
        <div className="text-center py-8 text-gray-500">Loading...</div>
      ) : plugins.length === 0 ? (
        <div className="text-center py-8">
          <Package className="w-12 h-12 mx-auto mb-2 text-gray-300" />
          <p className="text-gray-400">No plugins installed</p>
          <p className="text-sm text-gray-400 mt-1">
            Install plugins to extend functionality
          </p>
        </div>
      ) : (
        <div className="space-y-2">
          {plugins.map((plugin) => (
            <div
              key={plugin.id}
              className={`p-4 border rounded-lg ${
                plugin.enabled ? "border-blue-200 bg-blue-50/50 dark:bg-blue-900/10" : ""
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h4 className="font-medium">{plugin.name}</h4>
                    <span className="text-xs px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">
                      v{plugin.version}
                    </span>
                    {plugin.enabled && (
                      <span className="text-xs px-2 py-0.5 bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400 rounded">
                        Active
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-500 mt-1">
                    Installed: {new Date(plugin.installedAt).toLocaleDateString()}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() =>
                      plugin.enabled
                        ? disablePlugin(plugin.id)
                        : enablePlugin(plugin.id)
                    }
                    className={`p-2 rounded ${
                      plugin.enabled
                        ? "hover:bg-red-100 dark:hover:bg-red-900/30 text-red-500"
                        : "hover:bg-green-100 dark:hover:bg-green-900/30 text-green-500"
                    }`}
                    title={plugin.enabled ? "Disable" : "Enable"}
                  >
                    {plugin.enabled ? (
                      <PowerOff className="w-4 h-4" />
                    ) : (
                      <Power className="w-4 h-4" />
                    )}
                  </button>
                  <button
                    onClick={() => uninstallPlugin(plugin.id)}
                    className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-gray-500"
                    title="Uninstall"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Install Plugin */}
      <div className="mt-4 p-4 border border-dashed rounded-lg text-center">
        <p className="text-sm text-gray-500">
          Drop a plugin package here or click to browse
        </p>
        <button className="mt-2 px-4 py-2 text-sm bg-gray-100 dark:bg-gray-700 rounded hover:bg-gray-200 dark:hover:bg-gray-600">
          Install Plugin
        </button>
      </div>
    </div>
  );
}
