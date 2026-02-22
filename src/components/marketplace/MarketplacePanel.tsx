import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface MarketplaceItem {
  id: string;
  name: string;
  description: string;
  item_type: "skill" | "recipe" | "plugin" | "template";
  author: string;
  version: string;
  download_count: number;
  rating: number;
  price: string | { Paid: { amount: number; currency: string } };
  tags: string[];
  created_at: string;
  updated_at: string;
}

interface MarketplaceCategory {
  id: string;
  name: string;
  description: string;
  icon: string;
  item_count: number;
}

export const MarketplacePanel: React.FC = () => {
  const [items, setItems] = useState<MarketplaceItem[]>([]);
  const [categories, setCategories] = useState<MarketplaceCategory[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadItems = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<MarketplaceItem[]>("marketplace_list_items", {
        filters: null,
        page: 1,
        pageSize: 20,
      });
      setItems(result);
    } catch (err) {
      console.error("Failed to load marketplace items:", err);
      setError(`Failed to load items: ${err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadCategories = useCallback(async () => {
    try {
      const result = await invoke<MarketplaceCategory[]>(
        "marketplace_get_categories",
      );
      setCategories(result);
    } catch (err) {
      console.error("Failed to load categories:", err);
    }
  }, []);

  useEffect(() => {
    loadItems();
    loadCategories();
  }, [loadItems, loadCategories]);

  const getItemTypeIcon = (type: string) => {
    switch (type) {
      case "skill":
        return "⚡";
      case "recipe":
        return "📋";
      case "plugin":
        return "🔌";
      case "template":
        return "📄";
      default:
        return "📦";
    }
  };

  const getPriceText = (price: MarketplaceItem["price"]) => {
    if (typeof price === "string") return price;
    if ("Paid" in price) return `$${price.Paid.amount / 100}`;
    return "Free";
  };

  return (
    <div className="p-6 bg-white dark:bg-gray-800 min-h-full">
      <h2 className="text-xl font-bold mb-4">Marketplace</h2>

      {error && (
        <div className="p-4 mb-4 bg-red-100 text-red-700 rounded">{error}</div>
      )}

      {loading && <div className="text-center py-8">Loading...</div>}

      {!loading && items.length === 0 && !error && (
        <div className="text-center py-8 text-gray-500">No items found</div>
      )}

      {!loading && items.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {items.map((item) => (
            <div
              key={item.id}
              className="border dark:border-gray-700 rounded-lg p-4 hover:shadow-lg transition-shadow"
            >
              <div className="flex items-center gap-2 mb-2">
                <span className="text-2xl">
                  {getItemTypeIcon(item.item_type)}
                </span>
                <div>
                  <h3 className="font-semibold">{item.name}</h3>
                  <p className="text-sm text-gray-500">by {item.author}</p>
                </div>
              </div>
              <p className="text-sm text-gray-600 dark:text-gray-400 mb-2">
                {item.description}
              </p>
              <div className="flex items-center gap-2 text-sm">
                <span>⭐ {item.rating.toFixed(1)}</span>
                <span>⬇️ {item.download_count}</span>
                <span className="ml-auto font-semibold">
                  {getPriceText(item.price)}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
