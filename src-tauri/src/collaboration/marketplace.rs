//! Marketplace Client for Template Sharing
//!
//! This module provides a client for the template marketplace.

use crate::collaboration::Template;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Marketplace client for template operations
pub struct MarketplaceClient {
    base_url: String,
    api_key: Option<String>,
}

impl MarketplaceClient {
    /// Create a new marketplace client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            api_key: None,
        }
    }

    /// Create a client with API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// List templates from marketplace
    pub async fn list_templates(&self) -> Result<Vec<MarketplaceTemplate>, String> {
        // In production, this would make an HTTP request
        // For now, return mock data
        Ok(vec![
            MarketplaceTemplate {
                id: "mt-1".to_string(),
                name: "API Request Handler".to_string(),
                description: Some("Handle API requests with retry logic".to_string()),
                category: "development".to_string(),
                author: "Community".to_string(),
                version: "1.2.0".to_string(),
                downloads: 1500,
                rating: 4.5,
                tags: vec!["api".to_string(), "http".to_string()],
            },
            MarketplaceTemplate {
                id: "mt-2".to_string(),
                name: "Data Processing Pipeline".to_string(),
                description: Some("Process large datasets efficiently".to_string()),
                category: "data".to_string(),
                author: "Community".to_string(),
                version: "2.0.0".to_string(),
                downloads: 800,
                rating: 4.8,
                tags: vec!["data".to_string(), "pipeline".to_string()],
            },
        ])
    }

    /// Download a template from marketplace
    pub async fn download_template(&self, id: &str) -> Result<Template, String> {
        // In production, this would fetch from the marketplace
        // For now, return a mock template
        Ok(Template {
            id: id.to_string(),
            name: "Marketplace Template".to_string(),
            category: "imported".to_string(),
            content: "Imported template content".to_string(),
            visibility: "private".to_string(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Upload a template to marketplace
    pub async fn upload_template(&self, template: &Template) -> Result<String, String> {
        // In production, this would POST to the marketplace
        // For now, return a mock template ID
        Ok(format!("mp-{}", uuid::Uuid::new_v4()))
    }

    /// Search templates with filters
    pub async fn search_templates(
        &self,
        query: &str,
        category: Option<&str>,
        tags: Vec<&str>,
    ) -> Result<Vec<MarketplaceTemplate>, String> {
        let all_templates = self.list_templates().await?;

        let filtered: Vec<_> = all_templates
            .into_iter()
            .filter(|t| {
                let name_matches = t.name.to_lowercase().contains(&query.to_lowercase());
                let category_matches = category.map_or(true, |c| t.category == c);
                let tags_matches = tags.is_empty() || t.tags.iter().any(|tag| tags.contains(&tag.as_str()));
                name_matches && category_matches && tags_matches
            })
            .collect();

        Ok(filtered)
    }

    /// Get template details
    pub async fn get_template_details(&self, id: &str) -> Result<MarketplaceTemplate, String> {
        let templates = self.list_templates().await?;
        templates
            .into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Template not found: {}", id))
    }
}

/// Template from the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub author: String,
    pub version: String,
    pub downloads: u32,
    pub rating: f32,
    pub tags: Vec<String>,
}

/// Marketplace search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSearchFilters {
    pub category: Option<String>,
    pub min_rating: Option<f32>,
    pub tags: Vec<String>,
    pub author: Option<String>,
}

impl Default for MarketplaceSearchFilters {
    fn default() -> Self {
        Self {
            category: None,
            min_rating: None,
            tags: Vec::new(),
            author: None,
        }
    }
}

/// Marketplace upload options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceUploadOptions {
    pub publish_as: String,  // "self" or "organization"
    pub price: Option<f64>,  // For paid templates
    pub license: String,    // "MIT", "Apache-2.0", etc.
}

impl Default for MarketplaceUploadOptions {
    fn default() -> Self {
        Self {
            publish_as: "self".to_string(),
            price: None,
            license: "MIT".to_string(),
        }
    }
}

/// Cached marketplace data
pub struct MarketplaceCache {
    templates: HashMap<String, MarketplaceTemplate>,
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

impl MarketplaceCache {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            last_updated: None,
        }
    }

    pub fn insert(&mut self, template: MarketplaceTemplate) {
        let id = template.id.clone();
        self.templates.insert(id, template);
        self.last_updated = Some(chrono::Utc::now());
    }

    pub fn get(&self, id: &str) -> Option<&MarketplaceTemplate> {
        self.templates.get(id)
    }

    pub fn list(&self) -> Vec<&MarketplaceTemplate> {
        self.templates.values().collect()
    }

    pub fn clear(&mut self) {
        self.templates.clear();
        self.last_updated = None;
    }

    pub fn is_stale(&self, max_age_seconds: i64) -> bool {
        self.last_updated.map_or(true, |t| {
            chrono::Utc::now() - t > chrono::Duration::seconds(max_age_seconds)
        })
    }
}

impl Default for MarketplaceCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_templates() {
        let client = MarketplaceClient::new("https://api.marketplace.com".to_string());
        let result = client.list_templates().await;
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert!(!templates.is_empty());
    }

    #[tokio::test]
    async fn test_search_templates() {
        let client = MarketplaceClient::new("https://api.marketplace.com".to_string());
        let result = client.search_templates("api", Some("development"), vec![]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_is_stale() {
        let mut cache = MarketplaceCache::new();
        assert!(cache.is_stale(60)); // No data = stale

        cache.insert(MarketplaceTemplate {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            category: "test".to_string(),
            author: "Test".to_string(),
            version: "1.0.0".to_string(),
            downloads: 0,
            rating: 0.0,
            tags: vec![],
        });

        assert!(!cache.is_stale(3600)); // Fresh data
    }
}
