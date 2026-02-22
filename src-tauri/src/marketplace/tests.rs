// Marketplace integration tests

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;
    use crate::marketplace::{MarketplaceItem, MarketplaceItemType, MarketplacePrice, MarketplaceFilters, MarketplaceListing};

    #[tokio::test]
    async fn test_store_list_items() {
        let store = MarketplaceStore::default_marketplace();
        let filters = MarketplaceFilters::default();

        let items = store.list_items(&filters, 1, 20).await;
        assert!(items.is_ok());
        let items = items.unwrap();
        assert!(!items.is_empty());
    }

    #[tokio::test]
    async fn test_store_search_items() {
        let store = MarketplaceStore::default_marketplace();
        let filters = MarketplaceFilters::default();

        let items = store.search_items("code", &filters, 1, 20).await;
        assert!(items.is_ok());
        let items = items.unwrap();
        assert!(!items.is_empty());
    }

    #[tokio::test]
    async fn test_store_get_categories() {
        let store = MarketplaceStore::default_marketplace();

        let categories = store.get_categories().await;
        assert!(categories.is_ok());
        let categories = categories.unwrap();
        assert!(!categories.is_empty());
    }

    #[tokio::test]
    async fn test_store_get_item() {
        let store = MarketplaceStore::default_marketplace();

        let item = store.get_item("skill-code-reviewer").await;
        assert!(item.is_ok());
        let item = item.unwrap();
        assert_eq!(item.id, "skill-code-reviewer");
    }

    #[tokio::test]
    async fn test_listing_filter_by_type() {
        let mut listing = MarketplaceListing::new();

        let item = MarketplaceItem {
            id: "test-skill".to_string(),
            name: "Test Skill".to_string(),
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

        listing.add_item(item).unwrap();

        let skills = listing.filter_by_type(MarketplaceItemType::Skill);
        assert_eq!(skills.len(), 1);
    }

    #[tokio::test]
    async fn test_listing_search() {
        let mut listing = MarketplaceListing::new();

        let item = MarketplaceItem {
            id: "test-item".to_string(),
            name: "Code Reviewer".to_string(),
            description: "Reviews code".to_string(),
            item_type: MarketplaceItemType::Skill,
            author: "Test".to_string(),
            version: "1.0.0".to_string(),
            download_count: 0,
            rating: 0.0,
            price: MarketplacePrice::Free,
            tags: vec!["code".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        listing.add_item(item).unwrap();

        let results = listing.search("code");
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_listing_get_featured() {
        let mut listing = MarketplaceListing::new();

        for i in 0..5 {
            let item = MarketplaceItem {
                id: format!("item-{}", i),
                name: format!("Item {}", i),
                description: "Test".to_string(),
                item_type: MarketplaceItemType::Skill,
                author: "Test".to_string(),
                version: "1.0.0".to_string(),
                download_count: 100 - i as u64,
                rating: 4.0 + (i as f32) * 0.2,
                price: MarketplacePrice::Free,
                tags: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            listing.add_item(item).unwrap();
        }

        let featured = listing.get_featured(3);
        assert_eq!(featured.len(), 3);
    }
}
