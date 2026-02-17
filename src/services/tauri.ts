/**
 * Tauri API Service
 * Wraps Tauri commands for use in React
 */

import { invoke } from '@tauri-apps/api/core';

export interface FolderPermission {
  id: string;
  path: string;
  level: 'read' | 'readwrite';
}

/**
 * Get app version
 */
export async function getVersion(): Promise<string> {
  return invoke('get_version');
}

/**
 * Validate folder path
 */
export async function validateFolderPath(path: string): Promise<boolean> {
  return invoke('validate_folder_path', { path });
}

/**
 * Check folder access level
 */
export async function checkFolderAccess(
  path: string,
  permissions: FolderPermission[]
): Promise<string> {
  return invoke('check_folder_access', { path, permissions });
}

/**
 * Read file content
 */
export async function readFileContent(path: string): Promise<string> {
  return invoke('read_file_content', { path });
}

/**
 * Write file content
 */
export async function writeFileContent(path: string, content: string): Promise<void> {
  return invoke('write_file_content', { path, content });
}

/**
 * List directory contents
 */
export async function listDirectory(path: string): Promise<Array<{ name: string; isDir: boolean }>> {
  const result: string[] = await invoke('list_directory', { path });
  return result.map((entry) => {
    const [name, type] = entry.split(':');
    return { name, isDir: type === 'dir' };
  });
}
