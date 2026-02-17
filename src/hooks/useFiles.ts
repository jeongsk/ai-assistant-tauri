/**
 * Files Hook
 * Manages file system operations
 */

import { useState, useCallback } from 'react';
import { useSettingsStore } from '../stores/settingsStore';
import { 
  validateFolderPath, 
  checkFolderAccess, 
  readFileContent, 
  writeFileContent, 
  listDirectory
} from '../services/tauri';

export interface FileEntry {
  name: string;
  isDir: boolean;
}

export function useFiles() {
  const [currentPath, setCurrentPath] = useState<string>('');
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const folderPermissions = useSettingsStore((state) => state.folderPermissions);

  const navigateTo = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    
    try {
      // Check access
      await checkFolderAccess(path, folderPermissions);
      
      // List contents
      const entries = await listDirectory(path);
      setFiles(entries);
      setCurrentPath(path);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [folderPermissions]);

  const readFile = useCallback(async (path: string): Promise<string> => {
    await checkFolderAccess(path, folderPermissions);
    return readFileContent(path);
  }, [folderPermissions]);

  const writeFile = useCallback(async (path: string, content: string): Promise<void> => {
    await checkFolderAccess(path, folderPermissions);
    return writeFileContent(path, content);
  }, [folderPermissions]);

  return {
    currentPath,
    files,
    loading,
    error,
    navigateTo,
    readFile,
    writeFile,
  };
}
