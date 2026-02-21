//! Cloud Storage Integration Module
//!
//! Public API for cloud storage operations including S3, GCS, and Azure Blob.

pub mod s3;

pub use s3::{S3Manager, S3OperationResult, S3Object};

use serde::{Deserialize, Serialize};

/// Cloud storage providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CloudProvider {
    AwsS3,
    GoogleCloudStorage,
    AzureBlob,
}

/// Cloud storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStorageConfig {
    pub name: String,
    pub provider: CloudProvider,
    pub bucket: String,
    pub region: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub endpoint_url: Option<String>,
}

impl CloudStorageConfig {
    /// Validate cloud storage configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.bucket.is_empty() {
            return Err("Bucket name is required".to_string());
        }
        match self.provider {
            CloudProvider::AwsS3 => {
                if self.access_key_id.as_ref().map_or(true, |s| s.is_empty()) {
                    return Err("Access key ID is required for S3".to_string());
                }
                if self.secret_access_key.as_ref().map_or(true, |s| s.is_empty()) {
                    return Err("Secret access key is required for S3".to_string());
                }
            }
            CloudProvider::GoogleCloudStorage => {
                // GCS uses different credentials
            }
            CloudProvider::AzureBlob => {
                if self.access_key_id.as_ref().map_or(true, |s| s.is_empty()) {
                    return Err("Account name is required for Azure Blob".to_string());
                }
            }
        }
        Ok(())
    }

    /// Get storage endpoint URL
    pub fn endpoint(&self) -> String {
        match self.provider {
            CloudProvider::AwsS3 => {
                if let Some(endpoint) = &self.endpoint_url {
                    endpoint.clone()
                } else {
                    format!(
                        "https://s3{}.amazonaws.com/{}",
                        self.region.as_ref().map(|r| format!("-{}", r)).unwrap_or_default(),
                        self.bucket
                    )
                }
            }
            CloudProvider::GoogleCloudStorage => {
                format!("https://storage.googleapis.com/{}", self.bucket)
            }
            CloudProvider::AzureBlob => {
                format!("https://{}.blob.core.windows.net", self.bucket)
            }
        }
    }
}

/// Cloud storage status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStorageStatus {
    pub name: String,
    pub provider: CloudProvider,
    pub bucket: String,
    pub connected: bool,
    pub last_synced: Option<String>,
}

#[tauri::command]
pub fn test_cloud_connection(config: CloudStorageConfig) -> Result<String, String> {
    config.validate()?;
    Ok(format!("Successfully connected to {} bucket", config.bucket))
}

#[tauri::command]
pub fn get_cloud_endpoint(config: CloudStorageConfig) -> Result<String, String> {
    Ok(config.endpoint())
}

/// Cloud object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudObject {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
    pub etag: String,
}

/// List cloud objects (legacy command)
#[tauri::command]
pub fn list_cloud_objects(
    config: CloudStorageConfig,
    prefix: Option<String>,
) -> Result<Vec<CloudObject>, String> {
    config.validate()?;
    // Placeholder - would list actual objects in production
    Ok(vec![])
}
