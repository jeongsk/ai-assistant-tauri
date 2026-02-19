// Marketplace Install - Item installation and management

use crate::marketplace::MarketplaceItem;
use std::path::PathBuf;

/// Installation status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InstallationStatus {
    Installed,
    Pending,
    Failed,
    UpdateAvailable,
}

/// Installed item info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstalledItem {
    pub id: String,
    pub version: String,
    pub installed_at: String,
    pub status: InstallationStatus,
}

/// Marketplace installer
pub struct MarketplaceInstaller {
    install_dir: PathBuf,
    installed: std::collections::HashMap<String, InstalledItem>,
}

impl MarketplaceInstaller {
    /// Create a new installer
    pub fn new(install_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&install_dir)
            .map_err(|e| format!("Failed to create install directory: {}", e))?;

        Ok(Self {
            install_dir,
            installed: std::collections::HashMap::new(),
        })
    }

    /// Install an item from marketplace
    pub async fn install(&mut self, item: &MarketplaceItem) -> Result<String, String> {
        // Check if already installed
        if let Some(installed) = self.installed.get(&item.id) {
            if matches!(installed.status, InstallationStatus::Installed) {
                return Err(format!("Item already installed: {}", item.id));
            }
        }

        // Create installation record
        let installed_item = InstalledItem {
            id: item.id.clone(),
            version: item.version.clone(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            status: InstallationStatus::Installed,
        };

        // In production, this would:
        // 1. Download the item package
        // 2. Verify signature/checksum
        // 3. Extract to install directory
        // 4. Run installation scripts

        let item_path = self.get_item_path(&item.id);
        std::fs::create_dir_all(&item_path)
            .map_err(|e| format!("Failed to create item directory: {}", e))?;

        // Save metadata
        let metadata_path = item_path.join("metadata.json");
        let metadata = serde_json::to_string_pretty(&installed_item)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        std::fs::write(&metadata_path, metadata)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;

        self.installed.insert(item.id.clone(), installed_item);

        Ok(format!("Installed {} v{}", item.name, item.version))
    }

    /// Uninstall an item
    pub async fn uninstall(&mut self, item_id: &str) -> Result<String, String> {
        let item_path = self.get_item_path(item_id);

        if !item_path.exists() {
            return Err(format!("Item not installed: {}", item_id));
        }

        // Remove item files
        std::fs::remove_dir_all(&item_path)
            .map_err(|e| format!("Failed to remove item files: {}", e))?;

        self.installed.remove(item_id);

        Ok(format!("Uninstalled {}", item_id))
    }

    /// Check for updates
    pub async fn check_updates(&self) -> Result<Vec<String>, String> {
        let mut updates = Vec::new();

        for (id, installed) in &self.installed {
            // In production, this would check the marketplace API
            // For now, just return a mock update notification
            if id.starts_with("skill-") {
                updates.push(format!("{}: update available (v{} -> v{}.mock)",
                    id, installed.version, "2.0.0"));
            }
        }

        Ok(updates)
    }

    /// Update an item
    pub async fn update(&mut self, item_id: &str) -> Result<String, String> {
        if !self.installed.contains_key(item_id) {
            return Err(format!("Item not installed: {}", item_id));
        }

        // In production, this would:
        // 1. Download new version
        // 2. Backup current version
        // 3. Install new version
        // 4. Update metadata

        Ok(format!("Updated {}", item_id))
    }

    /// Get installed items
    pub fn get_installed(&self) -> Vec<&InstalledItem> {
        self.installed.values().collect()
    }

    /// Check if item is installed
    pub fn is_installed(&self, item_id: &str) -> bool {
        self.installed.contains_key(item_id)
    }

    /// Get item install path
    fn get_item_path(&self, item_id: &str) -> PathBuf {
        self.install_dir.join(item_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::{MarketplaceItemType, MarketplacePrice};

    #[tokio::test]
    async fn test_install_item() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut installer = MarketplaceInstaller::new(temp_dir.path().to_path_buf()).unwrap();

        let item = MarketplaceItem {
            id: "test-item".to_string(),
            name: "Test Item".to_string(),
            description: "Test".to_string(),
            item_type: MarketplaceItemType::Skill,
            author: "Test".to_string(),
            version: "1.0.0".to_string(),
            download_count: 0,
            rating: 0.0,
            price: MarketplacePrice::Free,
            tags: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let result = installer.install(&item).await;
        assert!(result.is_ok());
        assert!(installer.is_installed("test-item"));
    }

    #[tokio::test]
    async fn test_uninstall_item() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut installer = MarketplaceInstaller::new(temp_dir.path().to_path_buf()).unwrap();

        let item = MarketplaceItem {
            id: "test-item".to_string(),
            name: "Test Item".to_string(),
            description: "Test".to_string(),
            item_type: MarketplaceItemType::Skill,
            author: "Test".to_string(),
            version: "1.0.0".to_string(),
            download_count: 0,
            rating: 0.0,
            price: MarketplacePrice::Free,
            tags: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        installer.install(&item).await.unwrap();
        let result = installer.uninstall("test-item").await;
        assert!(result.is_ok());
        assert!(!installer.is_installed("test-item"));
    }
}
