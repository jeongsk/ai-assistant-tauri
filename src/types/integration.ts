/**
 * Integration Type Definitions
 */

export type DatabaseType = 'postgresql' | 'mysql';

export interface DatabaseConfig {
  type: DatabaseType;
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl?: boolean;
}

export interface GitConfig {
  repositoryPath: string;
  userName?: string;
  userEmail?: string;
}

export type CloudProvider = 's3' | 'gcs';

export interface CloudStorageConfig {
  provider: CloudProvider;
  bucket: string;
  region?: string;
  credentials?: {
    accessKeyId?: string;
    secretAccessKey?: string;
    projectId?: string;
    clientEmail?: string;
    privateKey?: string;
  };
}

export interface IntegrationStatus {
  name: string;
  type: 'database' | 'git' | 'cloud';
  connected: boolean;
  lastSync?: string;
  error?: string;
}
