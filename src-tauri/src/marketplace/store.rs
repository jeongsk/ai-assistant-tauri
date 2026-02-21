// Marketplace Store - Remote marketplace API client

use crate::marketplace::{MarketplaceItem, MarketplaceCategory, MarketplaceFilters};

/// Marketplace store client
pub struct MarketplaceStore {
    base_url: String,
    api_key: Option<String>,
}

impl MarketplaceStore {
    /// Create a new marketplace store client
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self { base_url, api_key }
    }

    /// Create with default official marketplace
    pub fn default_marketplace() -> Self {
        Self::new(
            "https://marketplace.ai-assistant.app/api".to_string(),
            None,
        )
    }

    /// List marketplace items with filters
    pub async fn list_items(
        &self,
        _filters: &MarketplaceFilters,
        _page: u32,
        _page_size: u32,
    ) -> Result<Vec<MarketplaceItem>, String> {
        // In production, this would make actual HTTP requests
        // For now, return mock data
        Ok(self.get_mock_items())
    }

    /// Get item details by ID
    pub async fn get_item(&self, item_id: &str) -> Result<MarketplaceItem, String> {
        let items = self.get_mock_items();
        items
            .into_iter()
            .find(|item| item.id == item_id)
            .ok_or_else(|| format!("Item not found: {}", item_id))
    }

    /// Get all categories
    pub async fn get_categories(&self) -> Result<Vec<MarketplaceCategory>, String> {
        Ok(vec![
            MarketplaceCategory {
                id: "productivity".to_string(),
                name: "Productivity".to_string(),
                description: "Boost your productivity".to_string(),
                icon: "⚡".to_string(),
                item_count: 42,
            },
            MarketplaceCategory {
                id: "development".to_string(),
                name: "Development".to_string(),
                description: "Tools for developers".to_string(),
                icon: "💻".to_string(),
                item_count: 38,
            },
            MarketplaceCategory {
                id: "communication".to_string(),
                name: "Communication".to_string(),
                description: "Email, chat, and more".to_string(),
                icon: "💬".to_string(),
                item_count: 25,
            },
            MarketplaceCategory {
                id: "creativity".to_string(),
                name: "Creativity".to_string(),
                description: "Creative writing and art".to_string(),
                icon: "🎨".to_string(),
                item_count: 31,
            },
        ])
    }

    /// Search items by query
    pub async fn search_items(
        &self,
        query: &str,
        _filters: &MarketplaceFilters,
        _page: u32,
        _page_size: u32,
    ) -> Result<Vec<MarketplaceItem>, String> {
        let items = self.get_mock_items();
        let query_lower = query.to_lowercase();

        Ok(items
            .into_iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&query_lower)
                    || item.description.to_lowercase().contains(&query_lower)
                    || item.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    /// Mock items for development
    fn get_mock_items(&self) -> Vec<MarketplaceItem> {
        let now = chrono::Utc::now().to_rfc3339();

        vec![
            MarketplaceItem {
                id: "skill-code-reviewer".to_string(),
                name: "Code Reviewer".to_string(),
                description: "Automated code review with best practices check".to_string(),
                item_type: crate::marketplace::MarketplaceItemType::Skill,
                author: "AI Assistant Team".to_string(),
                version: "1.2.0".to_string(),
                download_count: 12450,
                rating: 4.8,
                price: crate::marketplace::MarketplacePrice::Free,
                tags: vec!["development".to_string(), "code-quality".to_string()],
                created_at: now.clone(),
                updated_at: now.clone(),
            },
            MarketplaceItem {
                id: "recipe-meeting-notes".to_string(),
                name: "Meeting Notes Generator".to_string(),
                description: "Generate structured meeting notes from transcripts".to_string(),
                item_type: crate::marketplace::MarketplaceItemType::Recipe,
                author: "Productivity Labs".to_string(),
                version: "2.0.1".to_string(),
                download_count: 8932,
                rating: 4.6,
                price: crate::marketplace::MarketplacePrice::Free,
                tags: vec!["productivity".to_string(), "meetings".to_string()],
                created_at: now.clone(),
                updated_at: now.clone(),
            },
            MarketplaceItem {
                id: "plugin-github-integration".to_string(),
                name: "GitHub Integration".to_string(),
                description: "Seamless GitHub repository management".to_string(),
                item_type: crate::marketplace::MarketplaceItemType::Plugin,
                author: "DevTools Inc".to_string(),
                version: "1.5.0".to_string(),
                download_count: 5678,
                rating: 4.9,
                price: crate::marketplace::MarketplacePrice::Free,
                tags: vec!["development".to_string(), "git".to_string(), "github".to_string()],
                created_at: now.clone(),
                updated_at: now.clone(),
            },
            MarketplaceItem {
                id: "template-email-pro".to_string(),
                name: "Email Template Pro".to_string(),
                description: "Professional email templates for all occasions".to_string(),
                item_type: crate::marketplace::MarketplaceItemType::Template,
                author: "CommMaster".to_string(),
                version: "1.0.0".to_string(),
                download_count: 15234,
                rating: 4.7,
                price: crate::marketplace::MarketplacePrice::Paid {
                    amount: 499,
                    currency: "USD".to_string(),
                },
                tags: vec!["communication".to_string(), "email".to_string()],
                created_at: now.clone(),
                updated_at: now,
            },
        ]
    }
}

impl Default for MarketplaceStore {
    fn default() -> Self {
        Self::default_marketplace()
    }
}
