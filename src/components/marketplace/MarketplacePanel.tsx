import React, { useState, useEffect } from "react";
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
  price: { Free: {} } | { Paid: { amount: number; currency: string } };
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
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadItems();
    loadCategories();
  }, [selectedCategory, searchQuery]);

  const loadItems = async () => {
    setLoading(true);
    try {
      const result = await invoke<MarketplaceItem[]>("marketplace_list_items", {
        filters: {
          item_type: null,
          category: selectedCategory,
          price_free_only: false,
          min_rating: null,
          tags: [],
          author: null,
        },
        page: 1,
        page_size: 20,
      });
      setItems(result);
    } catch (error) {
      console.error("Failed to load marketplace items:", error);
    } finally {
      setLoading(false);
    }
  };

  const loadCategories = async () => {
    try {
      const result = await invoke<MarketplaceCategory[]>(
        "marketplace_get_categories",
      );
      setCategories(result);
    } catch (error) {
      console.error("Failed to load categories:", error);
    }
  };

  const installItem = async (itemId: string) => {
    try {
      await invoke("marketplace_install_item", { itemId });
      alert("Item installed successfully!");
    } catch (error) {
      console.error("Failed to install item:", error);
      alert(`Failed to install: ${error}`);
    }
  };

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
    if ("Free" in price) return "Free";
    return `$${price.Paid.amount / 100}`;
  };

  return (
    <div className="flex-1 flex flex-col bg-white dark:bg-gray-800 overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b dark:border-gray-700">
        <h2 className="text-xl font-bold mb-4">Marketplace</h2>

        {/* Search */}
        <div className="mb-4">
          <input
            type="text"
            placeholder="Search items..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2 border dark:border-gray-600 rounded-lg dark:bg-gray-700"
          />
        </div>

        {/* Categories */}
        <div className="flex gap-2 flex-wrap">
          <button
            onClick={() => setSelectedCategory(null)}
            className={`px-3 py-1 rounded-full text-sm ${
              selectedCategory === null
                ? "bg-blue-500 text-white"
                : "bg-gray-200 dark:bg-gray-700"
            }`}
          >
            All
          </button>
          {categories.map((category) => (
            <button
              key={category.id}
              onClick={() => setSelectedCategory(category.id)}
              className={`px-3 py-1 rounded-full text-sm ${
                selectedCategory === category.id
                  ? "bg-blue-500 text-white"
                  : "bg-gray-200 dark:bg-gray-700"
              }`}
            >
              {category.icon} {category.name}
            </button>
          ))}
        </div>
      </div>

      {/* Items Grid */}
      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="text-center py-8">Loading...</div>
        ) : items.length === 0 ? (
          <div className="text-center py-8 text-gray-500">No items found</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {items.map((item) => (
              <div
                key={item.id}
                className="border dark:border-gray-700 rounded-lg p-4 hover:shadow-lg transition-shadow"
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <span className="text-2xl">
                      {getItemTypeIcon(item.item_type)}
                    </span>
                    <div>
                      <h3 className="font-semibold">{item.name}</h3>
                      <p className="text-sm text-gray-500">by {item.author}</p>
                    </div>
                  </div>
                  <span className="text-sm bg-gray-200 dark:bg-gray-700 px-2 py-1 rounded">
                    {item.version}
                  </span>
                </div>

                <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                  {item.description}
                </p>

                <div className="flex items-center gap-4 mb-3 text-sm text-gray-500">
                  <div className="flex items-center gap-1">
                    <span>⭐</span>
                    <span>{item.rating.toFixed(1)}</span>
                  </div>
                  <div className="flex items-center gap-1">
                    <span>⬇️</span>
                    <span>{item.download_count.toLocaleString()}</span>
                  </div>
                  <div className="ml-auto font-semibold">
                    {getPriceText(item.price)}
                  </div>
                </div>

                <div className="flex gap-2 mb-3">
                  {item.tags.map((tag) => (
                    <span
                      key={tag}
                      className="text-xs bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-2 py-1 rounded"
                    >
                      {tag}
                    </span>
                  ))}
                </div>

                <button
                  onClick={() => installItem(item.id)}
                  className="w-full bg-blue-500 hover:bg-blue-600 text-white py-2 rounded-lg transition-colors"
                >
                  Install
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
