// Marketplace Module - Skill/Recipe/Plugin marketplace

pub mod store;
pub mod listing;
pub mod install;

#[cfg(test)]
mod tests;

// Re-export commonly used types from submodules
pub use store::MarketplaceStore;
pub use install::MarketplaceInstaller;


use serde::{Deserialize, Serialize};

/// Marketplace item type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MarketplaceItemType {
    Skill,
    Recipe,
    Plugin,
    Template,
}

/// Marketplace item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: MarketplaceItemType,
    pub author: String,
    pub version: String,
    pub download_count: u64,
    pub rating: f32,
    pub price: MarketplacePrice,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Marketplace pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketplacePrice {
    Free,
    Paid {
        amount: u64,
        currency: String,
    },
}

/// Marketplace category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub item_count: usize,
}

/// Marketplace search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceFilters {
    pub item_type: Option<MarketplaceItemType>,
    pub category: Option<String>,
    pub price_free_only: bool,
    pub min_rating: Option<f32>,
    pub tags: Vec<String>,
    pub author: Option<String>,
}

impl Default for MarketplaceFilters {
    fn default() -> Self {
        Self {
            item_type: None,
            category: None,
            price_free_only: false,
            min_rating: None,
            tags: Vec::new(),
            author: None,
        }
    }
}
