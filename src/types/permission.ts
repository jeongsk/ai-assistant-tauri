// Permission Types

export type PermissionLevel = 'read' | 'readwrite';

export interface FolderPermission {
  id: string;
  path: string;
  level: PermissionLevel;
  createdAt?: Date;
}

export interface PermissionState {
  permissions: FolderPermission[];
  isLoading: boolean;
  error: string | null;
}

export interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'dir';
  children?: FileNode[];
  permission?: PermissionLevel;
}

export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

export function hasPermission(
  path: string,
  permissions: FolderPermission[],
  requiredLevel: PermissionLevel
): boolean {
  for (const perm of permissions) {
    if (path.startsWith(perm.path)) {
      if (requiredLevel === 'read') {
        return perm.level === 'read' || perm.level === 'readwrite';
      }
      if (requiredLevel === 'readwrite') {
        return perm.level === 'readwrite';
      }
    }
  }
  return false;
}
