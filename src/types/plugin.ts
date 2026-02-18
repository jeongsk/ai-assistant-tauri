/**
 * Plugin Type Definitions
 */

export interface Plugin {
  id: string;
  name: string;
  version: string;
  manifest: string;
  permissions: string[];
  enabled: boolean;
  installedAt: string;
  updatedAt: string;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  main: string;
  permissions: PluginPermission[];
  apiVersion: string;
}

export type PluginPermission =
  | { type: 'fileSystem'; paths: string[]; access: 'read' | 'readwrite' }
  | { type: 'network'; hosts: string[] }
  | { type: 'database'; tables: string[] }
  | { type: 'system'; capabilities: string[] };

export interface PluginCreateInput {
  id: string;
  name: string;
  version: string;
  manifest: PluginManifest;
  permissions: PluginPermission[];
}
