//! AWS S3 Client Implementation
//!
//! Actual S3 operations using aws-sdk-s3.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "cloud")]
use aws_sdk_s3 as s3;

/// S3 operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3OperationResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// S3 object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Object {
    pub key: String,
    pub size: i64,
    pub last_modified: String,
    pub etag: String,
    pub storage_class: Option<String>,
}

/// S3 client wrapper
#[cfg(feature = "cloud")]
pub struct S3Client {
    config: aws_config::SdkConfig,
    bucket: String,
}

/// S3 manager for multiple S3 connections
pub struct S3Manager {
    clients: Arc<RwLock<std::collections::HashMap<String, Arc<S3ClientWrapper>>>>,
}

/// Wrapper that works with and without the cloud feature
#[cfg(feature = "cloud")]
struct S3ClientWrapper {
    bucket: String,
    region: Option<String>,
    client: Option<Arc<s3::Client>>,
}

/// Wrapper for non-cloud builds
#[cfg(not(feature = "cloud"))]
struct S3ClientWrapper {
    bucket: String,
    region: Option<String>,
}

#[cfg(feature = "cloud")]
impl S3ClientWrapper {
    /// Get or initialize the S3 client
    #[cfg(feature = "cloud")]
    async fn get_or_init_client(&self) -> Result<Arc<s3::Client>, String> {
        if let Some(ref client) = self.client {
            return Ok(client.clone());
        }

        // Initialize AWS config
        let region_str = self.region.as_deref().unwrap_or("us-east-1");
        let region = s3::config::Region::new(region_str.to_string());

        let config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region)
            .load()
            .await;

        let client = Arc::new(s3::Client::new(&config_loader));
        Ok(client)
    }

    /// Get bucket name
    fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[cfg(not(feature = "cloud"))]
impl S3ClientWrapper {
    /// Get bucket name
    fn bucket(&self) -> &str {
        &self.bucket
    }
}

impl S3Manager {
    /// Create a new S3 manager
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add an S3 client configuration
    #[cfg(feature = "cloud")]
    pub async fn add_client(
        &self,
        name: String,
        bucket: String,
        region: Option<String>,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
    ) -> Result<(), String> {
        // If credentials provided, set them as environment variables for the SDK
        if let (Some(akid), Some(sak)) = (access_key_id, secret_access_key) {
            std::env::set_var("AWS_ACCESS_KEY_ID", akid);
            std::env::set_var("AWS_SECRET_ACCESS_KEY", sak);
        }

        let wrapper = S3ClientWrapper {
            bucket,
            region,
            client: None, // Will be initialized on first use
        };

        let mut clients = self.clients.write().await;
        clients.insert(name, Arc::new(wrapper));
        Ok(())
    }

    /// Add an S3 client configuration (non-cloud feature)
    #[cfg(not(feature = "cloud"))]
    pub async fn add_client(
        &self,
        name: String,
        bucket: String,
        region: Option<String>,
        _access_key_id: Option<String>,
        _secret_access_key: Option<String>,
    ) -> Result<(), String> {
        let wrapper = S3ClientWrapper {
            bucket,
            region,
        };

        let mut clients = self.clients.write().await;
        clients.insert(name, Arc::new(wrapper));
        Ok(())
    }

    /// Upload a file to S3
    #[cfg(feature = "cloud")]
    pub async fn upload_file(
        &self,
        name: &str,
        key: &str,
        data: Vec<u8>,
    ) -> S3OperationResult {
        use std::time::Instant;

        let start = Instant::now();

        let clients = self.clients.read().await;
        let wrapper = clients.get(name);

        let wrapper = match wrapper {
            Some(w) => w,
            None => {
                return S3OperationResult {
                    success: false,
                    result: None,
                    error: Some(format!("S3 client '{}' not found", name)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let client = match wrapper.get_or_init_client().await {
            Ok(c) => c,
            Err(e) => {
                return S3OperationResult {
                    success: false,
                    result: None,
                    error: Some(format!("Failed to initialize S3 client: {}", e)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let bucket = wrapper.bucket();

        // Execute upload
        match client.put_object()
            .bucket(bucket)
            .key(key)
            .body(aws_sdk_s3::primitives::ByteStream::from(data))
            .send()
            .await {
            Ok(output) => {
                let version_id = output.version_id().map(|v| v.to_string());
                let result_str = if let Some(v) = version_id {
                    format!("Uploaded {} to {} (version: {})", key, bucket, v)
                } else {
                    format!("Uploaded {} to {}", key, bucket)
                };

                S3OperationResult {
                    success: true,
                    result: Some(result_str),
                    error: None,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(e) => S3OperationResult {
                success: false,
                result: None,
                error: Some(format!("Upload failed: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Download a file from S3
    #[cfg(feature = "cloud")]
    pub async fn download_file(&self, name: &str, key: &str) -> S3OperationResult {
        use std::time::Instant;

        let start = Instant::now();

        let clients = self.clients.read().await;
        let wrapper = clients.get(name);

        let wrapper = match wrapper {
            Some(w) => w,
            None => {
                return S3OperationResult {
                    success: false,
                    result: None,
                    error: Some(format!("S3 client '{}' not found", name)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let client = match wrapper.get_or_init_client().await {
            Ok(c) => c,
            Err(e) => {
                return S3OperationResult {
                    success: false,
                    result: None,
                    error: Some(format!("Failed to initialize S3 client: {}", e)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let bucket = wrapper.bucket();

        // Execute download
        match client.get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await {
            Ok(output) => {
                // Try to collect the body
                match output.body.collect().await {
                    Ok(data) => {
                        let bytes = data.into_bytes().to_vec();
                        S3OperationResult {
                            success: true,
                            result: Some(format!("Downloaded {} bytes from {}", bytes.len(), key)),
                            error: None,
                            execution_time_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                    Err(e) => S3OperationResult {
                        success: false,
                        result: None,
                        error: Some(format!("Failed to read response body: {}", e)),
                        execution_time_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }
            Err(e) => S3OperationResult {
                success: false,
                result: None,
                error: Some(format!("Download failed: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// List objects in S3 bucket
    #[cfg(feature = "cloud")]
    pub async fn list_objects(&self, name: &str, prefix: Option<&str>) -> Result<Vec<S3Object>, String> {
        let clients = self.clients.read().await;
        let wrapper = clients.get(name)
            .ok_or_else(|| format!("S3 client '{}' not found", name))?;

        let client = wrapper.get_or_init_client().await?;
        let bucket = wrapper.bucket();

        let mut list_request = client.list_objects_v2()
            .bucket(bucket)
            .max_keys(1000);

        if let Some(p) = prefix {
            list_request = list_request.prefix(p);
        }

        let response = list_request.send()
            .await
            .map_err(|e| format!("List objects failed: {}", e))?;

        let objects = response.contents.unwrap_or_default();

        let mut result = Vec::new();
        for obj in objects {
            result.push(S3Object {
                key: obj.key.as_deref().unwrap_or("").to_string(),
                size: obj.size.unwrap_or(0),
                last_modified: obj.last_modified
                    .as_ref()
                    .map(|d| d.as_secs_f64().to_string())
                    .unwrap_or_default(),
                etag: obj.e_tag.as_deref().unwrap_or("").to_string(),
                storage_class: obj.storage_class.as_ref().map(|sc| sc.as_str().to_string()),
            });
        }

        Ok(result)
    }

    /// Delete an object from S3
    #[cfg(feature = "cloud")]
    pub async fn delete_object(&self, name: &str, key: &str) -> S3OperationResult {
        use std::time::Instant;

        let start = Instant::now();

        let clients = self.clients.read().await;
        let wrapper = clients.get(name);

        let wrapper = match wrapper {
            Some(w) => w,
            None => {
                return S3OperationResult {
                    success: false,
                    result: None,
                    error: Some(format!("S3 client '{}' not found", name)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let client = match wrapper.get_or_init_client().await {
            Ok(c) => c,
            Err(e) => {
                return S3OperationResult {
                    success: false,
                    result: None,
                    error: Some(format!("Failed to initialize S3 client: {}", e)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let bucket = wrapper.bucket();

        // Execute delete
        match client.delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await {
            Ok(_) => S3OperationResult {
                success: true,
                result: Some(format!("Deleted {} from {}", key, bucket)),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => S3OperationResult {
                success: false,
                result: None,
                error: Some(format!("Delete failed: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Remove an S3 client
    pub async fn remove_client(&self, name: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(name);
    }

    /// List all S3 clients
    pub async fn list_clients(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }
}

impl Default for S3Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "cloud"))]
impl S3Manager {
    pub async fn upload_file(&self, _name: &str, _key: &str, _data: Vec<u8>) -> S3OperationResult {
        S3OperationResult {
            success: false,
            result: None,
            error: Some("Cloud feature not enabled. Build with --features cloud".to_string()),
            execution_time_ms: 0,
        }
    }

    pub async fn download_file(&self, _name: &str, _key: &str) -> S3OperationResult {
        S3OperationResult {
            success: false,
            result: None,
            error: Some("Cloud feature not enabled. Build with --features cloud".to_string()),
            execution_time_ms: 0,
        }
    }

    pub async fn list_objects(&self, name: &str, _prefix: Option<&str>) -> Result<Vec<S3Object>, String> {
        Err(format!("Cloud feature not enabled for '{}'", name))
    }

    pub async fn delete_object(&self, _name: &str, _key: &str) -> S3OperationResult {
        S3OperationResult {
            success: false,
            result: None,
            error: Some("Cloud feature not enabled. Build with --features cloud".to_string()),
            execution_time_ms: 0,
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Upload file to S3
#[tauri::command]
pub async fn s3_upload(
    name: String,
    key: String,
    data: Vec<u8>,
) -> std::result::Result<S3OperationResult, String> {
    let manager = S3Manager::new();
    Ok(manager.upload_file(&name, &key, data).await)
}

/// Download file from S3
#[tauri::command]
pub async fn s3_download(
    name: String,
    key: String,
) -> std::result::Result<S3OperationResult, String> {
    let manager = S3Manager::new();
    Ok(manager.download_file(&name, &key).await)
}

/// List S3 objects
#[tauri::command]
pub async fn s3_list(
    name: String,
    prefix: Option<String>,
) -> std::result::Result<Vec<S3Object>, String> {
    let manager = S3Manager::new();
    manager.list_objects(&name, prefix.as_deref()).await
}

/// Delete S3 object
#[tauri::command]
pub async fn s3_delete(
    name: String,
    key: String,
) -> std::result::Result<S3OperationResult, String> {
    let manager = S3Manager::new();
    Ok(manager.delete_object(&name, &key).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_operation_result_serialization() {
        let result = S3OperationResult {
            success: true,
            result: Some("Uploaded file".to_string()),
            error: None,
            execution_time_ms: 100,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Uploaded file"));
    }

    #[test]
    fn test_s3_object_serialization() {
        let obj = S3Object {
            key: "test.txt".to_string(),
            size: 1024,
            last_modified: "2024-01-01T00:00:00Z".to_string(),
            etag: "\"abc\"".to_string(),
            storage_class: Some("STANDARD".to_string()),
        };

        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("test.txt"));
    }

    #[tokio::test]
    async fn test_s3_manager_new() {
        let manager = S3Manager::new();
        let clients = manager.list_clients().await;
        assert!(clients.is_empty());
    }
}
