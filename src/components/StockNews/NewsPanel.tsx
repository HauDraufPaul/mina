import { useState } from "react";
import { useStockNews } from "../../hooks/useStockNews";
import { useStockNewsStore } from "../../stores/stockNewsStore";
import NewsItem from "./NewsItem";
import NewsFilter from "./NewsFilter";
import { RefreshCw, Search, X } from "lucide-react";
import Button from "../ui/Button";

export default function NewsPanel() {
  const { refreshNews, loading, fetchNews } = useStockNews({ autoFetch: true, autoSubscribe: true });
  const { getFilteredNews, searchQuery, setSearchQuery, newsItems } = useStockNewsStore();
  const [localSearch, setLocalSearch] = useState("");
  const [refreshing, setRefreshing] = useState(false);

  const filteredNews = getFilteredNews();

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      console.log("Refreshing news...");
      const count = await refreshNews();
      console.log(`Refresh complete, fetched ${count} items`);
      // Also fetch the news to update the UI
      await fetchNews();
    } catch (err) {
      console.error("Refresh failed:", err);
    } finally {
      setTimeout(() => setRefreshing(false), 1000);
    }
  };

  const handleSearch = () => {
    setSearchQuery(localSearch);
  };

  const clearSearch = () => {
    setLocalSearch("");
    setSearchQuery("");
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-white/10">
        <div className="flex items-center gap-3">
          <h2 className="text-xl font-bold text-white font-mono">
            STOCK NEWS
          </h2>
          <span className="text-sm text-gray-400 font-mono">
            {filteredNews.length} items
          </span>
        </div>

        <div className="flex items-center gap-2">
          <Button
            onClick={handleRefresh}
            disabled={refreshing}
            variant="ghost"
            className="p-2"
          >
            <RefreshCw
              className={`w-4 h-4 ${refreshing ? "animate-spin" : ""}`}
            />
          </Button>
        </div>
      </div>

      {/* Search Bar */}
      <div className="p-4 border-b border-white/10">
        <div className="flex gap-2">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              value={localSearch}
              onChange={(e) => setLocalSearch(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              placeholder="Search news..."
              className="w-full bg-white/5 border border-white/10 rounded px-10 py-2 text-sm
                       text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
            {searchQuery && (
              <button
                onClick={clearSearch}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 
                         hover:text-white transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
          <Button onClick={handleSearch} className="px-4 py-2">
            Search
          </Button>
        </div>
      </div>

      {/* Filters */}
      <NewsFilter />

      {/* News List */}
      <div className="flex-1 overflow-y-auto">
        {loading && filteredNews.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <RefreshCw className="w-8 h-8 text-neon-cyan animate-spin mx-auto mb-2" />
              <p className="text-gray-400 font-mono">Loading news...</p>
              <p className="text-xs text-gray-500 mt-2">
                Fetching from RSS feeds...
              </p>
            </div>
          </div>
        ) : filteredNews.length === 0 && newsItems.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center space-y-4">
              <p className="text-gray-400 font-mono">No news available yet</p>
              <p className="text-sm text-gray-500">
                The system is polling RSS feeds every 60 seconds.
              </p>
              <p className="text-sm text-gray-500">
                Click the refresh button above to fetch news immediately.
              </p>
              <Button onClick={handleRefresh} disabled={refreshing}>
                {refreshing ? "Fetching..." : "Fetch News Now"}
              </Button>
            </div>
          </div>
        ) : filteredNews.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <p className="text-gray-400 font-mono">No news found</p>
              <p className="text-sm text-gray-500 mt-2">
                Try adjusting your filters or clearing them
              </p>
            </div>
          </div>
        ) : (
          <div className="divide-y divide-white/5">
            {filteredNews.map((item) => (
              <NewsItem key={item.id} item={item} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

