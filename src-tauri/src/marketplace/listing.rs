// Marketplace Listing - Item listing and management

use crate::marketplace::{MarketplaceItem, MarketplaceItemType};
use std::collections::HashMap;

/// Marketplace listing manager
pub struct MarketplaceListing {
    items: HashMap<String, MarketplaceItem>,
}

impl MarketplaceListing {
    /// Create a new listing manager
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Add item to listing
    pub fn add_item(&mut self, item: MarketplaceItem) -> Result<(), String> {
        if item.id.is_empty() {
            return Err("Item ID is required".to_string());
        }
        self.items.insert(item.id.clone(), item);
        Ok(())
    }

    /// Remove item from listing
    pub fn remove_item(&mut self, item_id: &str) -> Result<(), String> {
        self.items
            .remove(item_id)
            .map(|_| ())
            .ok_or_else(|| format!("Item not found: {}", item_id))
    }

    /// Get item by ID
    pub fn get_item(&self, item_id: &str) -> Option<&MarketplaceItem> {
        self.items.get(item_id)
    }

    /// List all items
    pub fn list_items(&self) -> Vec<&MarketplaceItem> {
        self.items.values().collect()
    }

    /// Filter items by type
    pub fn filter_by_type(&self, item_type: MarketplaceItemType) -> Vec<&MarketplaceItem> {
        self.items
            .values()
            .filter(|item| item.item_type == item_type)
            .collect()
    }

    /// Filter items by tags
    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<&MarketplaceItem> {
        self.items
            .values()
            .filter(|item| {
                tags.iter()
                    .all(|tag| item.tags.contains(tag))
            })
            .collect()
    }

    /// Filter items by author
    pub fn filter_by_author(&self, author: &str) -> Vec<&MarketplaceItem> {
        self.items
            .values()
            .filter(|item| item.author == author)
            .collect()
    }

    /// Search items by query
    pub fn search(&self, query: &str) -> Vec<&MarketplaceItem> {
        let query_lower = query.to_lowercase();
        self.items
            .values()
            .filter(|item| {
                item.name.to_lowercase().contains(&query_lower)
                    || item.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get featured items
    pub fn get_featured(&self, limit: usize) -> Vec<&MarketplaceItem> {
        let mut items: Vec<_> = self.items.values().collect();
        items.sort_by(|a, b| {
            b.download_count
                .cmp(&a.download_count)
                .then(b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal))
        });
        items.into_iter().take(limit).collect()
    }

    /// Get items by price (free only)
    pub fn get_free_items(&self) -> Vec<&MarketplaceItem> {
        self.items
            .values()
            .filter(|item| matches!(item.price, crate::marketplace::MarketplacePrice::Free))
            .collect()
    }
}

impl Default for MarketplaceListing {
    fn default() -> Self {
        Self::new()
    }
}
