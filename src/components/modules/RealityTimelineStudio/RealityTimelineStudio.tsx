import { useState, useEffect, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import Modal from "../../ui/Modal";
import { 
  Rss, 
  Plus, 
  Link, 
  Search, 
  Filter, 
  Edit2, 
  Trash2, 
  Star,
  StarOff,
  Loader2,
  ExternalLink,
  Clock,
  RefreshCw
} from "lucide-react";

interface RSSFeed {
  id: number;
  url: string;
  name: string;
  enabled: boolean;
  reliability: number;
  last_fetch?: number;
  created_at: number;
}

interface RSSItem {
  id: number;
  feed_id: number;
  title: string;
  content: string;
  url: string;
  published_at: number;
  fetched_at: number;
}

export default function RealityTimelineStudio() {
  const [feeds, setFeeds] = useState<RSSFeed[]>([]);
  const [items, setItems] = useState<RSSItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingItems, setLoadingItems] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterEnabled, setFilterEnabled] = useState<"all" | "enabled" | "disabled">("all");
  const [showAddModal, setShowAddModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState<RSSFeed | null>(null);
  const [newFeedUrl, setNewFeedUrl] = useState("");
  const [newFeedName, setNewFeedName] = useState("");
  const [newFeedReliability, setNewFeedReliability] = useState(0.5);
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState("");
  const searchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounce search
  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
    }
    searchTimeoutRef.current = setTimeout(() => {
      setDebouncedSearchQuery(searchQuery);
    }, 300);

    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [searchQuery]);

  useEffect(() => {
    loadFeeds();
    loadItems();
  }, []);

  const loadFeeds = async () => {
    try {
      setLoading(true);
      const feedsData = await invoke<RSSFeed[]>("list_rss_feeds");
      setFeeds(feedsData);
    } catch (error) {
      console.error("Failed to load feeds:", error);
    } finally {
      setLoading(false);
    }
  };

  const loadItems = async () => {
    try {
      setLoadingItems(true);
      const itemsData = await invoke<RSSItem[]>("get_recent_rss_items", { limit: 100 });
      setItems(itemsData);
    } catch (error) {
      console.error("Failed to load items:", error);
    } finally {
      setLoadingItems(false);
    }
  };

  const handleCreateFeed = async () => {
    if (!newFeedUrl.trim() || !newFeedName.trim()) {
      alert("Please enter both URL and name");
      return;
    }

    try {
      await invoke("create_rss_feed", {
        url: newFeedUrl,
        name: newFeedName,
        reliability: newFeedReliability,
      });
      setNewFeedUrl("");
      setNewFeedName("");
      setNewFeedReliability(0.5);
      setShowAddModal(false);
      await loadFeeds();
    } catch (error) {
      alert(`Failed to create feed: ${error}`);
    }
  };

  const handleUpdateFeed = async (feed: RSSFeed, updates: Partial<RSSFeed>) => {
    try {
      await invoke("update_rss_feed", {
        id: feed.id,
        name: updates.name,
        url: updates.url,
        reliability: updates.reliability,
        enabled: updates.enabled,
      });
      setShowEditModal(null);
      await loadFeeds();
    } catch (error) {
      alert(`Failed to update feed: ${error}`);
    }
  };

  const handleDeleteFeed = async (id: number) => {
    if (!confirm("Are you sure you want to delete this feed?")) {
      return;
    }

    try {
      await invoke("delete_rss_feed", { id });
      await loadFeeds();
      await loadItems();
    } catch (error) {
      alert(`Failed to delete feed: ${error}`);
    }
  };

  const handleToggleEnabled = async (feed: RSSFeed) => {
    await handleUpdateFeed(feed, { enabled: !feed.enabled });
  };

  // Filtered feeds
  const filteredFeeds = useMemo(() => {
    let filtered = feeds;

    // Apply enabled filter
    if (filterEnabled === "enabled") {
      filtered = filtered.filter(f => f.enabled);
    } else if (filterEnabled === "disabled") {
      filtered = filtered.filter(f => !f.enabled);
    }

    // Apply search
    if (debouncedSearchQuery.trim()) {
      const query = debouncedSearchQuery.toLowerCase();
      filtered = filtered.filter(f => 
        f.name.toLowerCase().includes(query) ||
        f.url.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [feeds, debouncedSearchQuery, filterEnabled]);

  // Filtered items
  const filteredItems = useMemo(() => {
    let filtered = items;

    // Apply search
    if (debouncedSearchQuery.trim()) {
      const query = debouncedSearchQuery.toLowerCase();
      filtered = filtered.filter(item => 
        item.title.toLowerCase().includes(query) ||
        item.content.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [items, debouncedSearchQuery]);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (days > 0) {
      return `${days}d ago`;
    } else if (hours > 0) {
      return `${hours}h ago`;
    } else {
      const minutes = Math.floor(diff / (1000 * 60));
      return minutes > 0 ? `${minutes}m ago` : "Just now";
    }
  };

  const getReliabilityColor = (reliability: number) => {
    if (reliability >= 0.8) return "text-neon-green";
    if (reliability >= 0.6) return "text-neon-cyan";
    if (reliability >= 0.4) return "text-neon-amber";
    return "text-gray-400";
  };

  const getReliabilityBadge = (reliability: number) => {
    if (reliability >= 0.8) return "Highly Reliable";
    if (reliability >= 0.6) return "Reliable";
    if (reliability >= 0.4) return "Moderate";
    return "Low";
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <Loader2 className="w-8 h-8 mx-auto mb-4 text-neon-cyan animate-spin" />
          <p className="text-gray-400">Loading Reality & Timeline Studio...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6 relative overflow-hidden">
      {/* Space background effect */}
      <div className="fixed inset-0 pointer-events-none -z-10">
        <div className="absolute inset-0 bg-black">
          {[...Array(50)].map((_, i) => (
            <div
              key={i}
              className="absolute w-1 h-1 bg-white rounded-full animate-pulse"
              style={{
                left: `${Math.random() * 100}%`,
                top: `${Math.random() * 100}%`,
                animationDelay: `${Math.random() * 3}s`,
                animationDuration: `${2 + Math.random() * 2}s`,
                opacity: Math.random() * 0.8 + 0.2,
              }}
            />
          ))}
        </div>
      </div>

      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
            Reality & Timeline Studio
          </h1>
          <p className="text-gray-400">OSINT Intelligence Hub</p>
        </div>
        <Button variant="primary" onClick={() => setShowAddModal(true)}>
          <Plus className="w-4 h-4 mr-2" />
          Add RSS Source
        </Button>
      </div>

      {/* Search and Filter */}
      <Card>
        <div className="flex gap-4 items-center">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search feeds and articles..."
              className="glass-input w-full pl-10 pr-10"
            />
            {searchQuery !== debouncedSearchQuery && (
              <div className="absolute right-10 top-1/2 transform -translate-y-1/2">
                <Loader2 className="w-3 h-3 text-gray-400 animate-spin" />
              </div>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Filter className="w-4 h-4 text-gray-400" />
            <select
              value={filterEnabled}
              onChange={(e) => setFilterEnabled(e.target.value as "all" | "enabled" | "disabled")}
              className="glass-input"
            >
              <option value="all">All Feeds</option>
              <option value="enabled">Enabled</option>
              <option value="disabled">Disabled</option>
            </select>
          </div>
          <Button variant="secondary" onClick={loadItems}>
            <RefreshCw className="w-4 h-4 mr-2" />
            Refresh
          </Button>
        </div>
      </Card>

      {/* RSS Sources */}
      <Card title="RSS Sources" className="relative">
        <div className="space-y-3">
          {filteredFeeds.length === 0 ? (
            <div className="text-center text-gray-400 py-12">
              <Rss className="w-16 h-16 mx-auto mb-4 text-gray-500 opacity-50" />
              <p className="text-lg mb-2">No RSS feeds configured</p>
              <p className="text-sm">Add your first RSS source to start monitoring</p>
            </div>
          ) : (
            filteredFeeds.map((feed) => (
              <div
                key={feed.id}
                className="glass-card p-4 hover:border-neon-cyan/50 transition-all"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-2">
                      <Rss className="w-5 h-5 text-neon-cyan flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-semibold text-lg truncate">{feed.name}</span>
                          <span
                            className={`text-xs px-2 py-0.5 rounded ${getReliabilityColor(feed.reliability)} bg-opacity-20`}
                          >
                            {getReliabilityBadge(feed.reliability)}
                          </span>
                          <div className="flex items-center gap-1">
                            {[...Array(5)].map((_, i) => (
                              <Star
                                key={i}
                                className={`w-3 h-3 ${
                                  i < Math.round(feed.reliability * 5)
                                    ? "text-neon-amber fill-neon-amber"
                                    : "text-gray-600"
                                }`}
                              />
                            ))}
                          </div>
                        </div>
                        <div className="text-sm text-gray-400 truncate mt-1">{feed.url}</div>
                        {feed.last_fetch && (
                          <div className="text-xs text-gray-500 mt-1 flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            Last fetch: {formatTime(feed.last_fetch)}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 ml-4">
                    <button
                      onClick={() => handleToggleEnabled(feed)}
                      className={`p-2 rounded transition-colors ${
                        feed.enabled
                          ? "bg-neon-green/20 text-neon-green hover:bg-neon-green/30"
                          : "bg-gray-500/20 text-gray-500 hover:bg-gray-500/30"
                      }`}
                      title={feed.enabled ? "Disable feed" : "Enable feed"}
                    >
                      {feed.enabled ? <Star className="w-4 h-4" /> : <StarOff className="w-4 h-4" />}
                    </button>
                    <button
                      onClick={() => setShowEditModal(feed)}
                      className="p-2 rounded bg-neon-cyan/20 text-neon-cyan hover:bg-neon-cyan/30 transition-colors"
                      title="Edit feed"
                    >
                      <Edit2 className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handleDeleteFeed(feed.id)}
                      className="p-2 rounded bg-red-500/20 text-red-500 hover:bg-red-500/30 transition-colors"
                      title="Delete feed"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </Card>

      {/* Top 100 Articles */}
      <Card title={`Top Articles (${filteredItems.length})`} className="relative">
        {loadingItems ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-8 h-8 text-neon-cyan animate-spin" />
          </div>
        ) : filteredItems.length === 0 ? (
          <div className="text-center text-gray-400 py-12">
            <Link className="w-16 h-16 mx-auto mb-4 text-gray-500 opacity-50" />
            <p className="text-lg mb-2">No articles found</p>
            <p className="text-sm">Articles will appear here once feeds are configured and fetched</p>
          </div>
        ) : (
          <div className="space-y-4">
            {filteredItems.map((item) => {
              const feed = feeds.find(f => f.id === item.feed_id);
              return (
                <div
                  key={item.id}
                  className="glass-card p-4 hover:border-neon-cyan/50 transition-all group"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        {feed && (
                          <span className="text-xs px-2 py-0.5 rounded bg-neon-cyan/20 text-neon-cyan">
                            {feed.name}
                          </span>
                        )}
                        <span className="text-xs text-gray-500 flex items-center gap-1">
                          <Clock className="w-3 h-3" />
                          {formatTime(item.published_at)}
                        </span>
                      </div>
                      <h3 className="font-semibold text-lg mb-2 group-hover:text-neon-cyan transition-colors">
                        {item.title}
                      </h3>
                      <p className="text-sm text-gray-300 mb-3 line-clamp-2">{item.content}</p>
                      <a
                        href={item.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-2 text-sm text-neon-cyan hover:text-neon-cyan/80 transition-colors"
                      >
                        <ExternalLink className="w-4 h-4" />
                        Read Article
                      </a>
                    </div>
                    {feed && (
                      <div className="flex flex-col items-center gap-1">
                        <div className={`text-xs ${getReliabilityColor(feed.reliability)}`}>
                          {Math.round(feed.reliability * 100)}%
                        </div>
                        <div className="flex flex-col gap-0.5">
                          {[...Array(5)].map((_, i) => (
                            <Star
                              key={i}
                              className={`w-2 h-2 ${
                                i < Math.round(feed.reliability * 5)
                                  ? "text-neon-amber fill-neon-amber"
                                  : "text-gray-600"
                              }`}
                            />
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </Card>

      {/* Add Feed Modal */}
      <Modal
        isOpen={showAddModal}
        onClose={() => {
          setShowAddModal(false);
          setNewFeedUrl("");
          setNewFeedName("");
          setNewFeedReliability(0.5);
        }}
        title="Add RSS Source"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Source Name</label>
            <input
              type="text"
              value={newFeedName}
              onChange={(e) => setNewFeedName(e.target.value)}
              placeholder="e.g., TechCrunch"
              className="glass-input w-full"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">RSS Feed URL</label>
            <input
              type="text"
              value={newFeedUrl}
              onChange={(e) => setNewFeedUrl(e.target.value)}
              placeholder="https://example.com/feed.xml"
              className="glass-input w-full"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">
              Reliability: {Math.round(newFeedReliability * 100)}%
            </label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={newFeedReliability}
              onChange={(e) => setNewFeedReliability(parseFloat(e.target.value))}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>Low</span>
              <span>High</span>
            </div>
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowAddModal(false);
                setNewFeedUrl("");
                setNewFeedName("");
                setNewFeedReliability(0.5);
              }}
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={handleCreateFeed}>
              Add Source
            </Button>
          </div>
        </div>
      </Modal>

      {/* Edit Feed Modal */}
      {showEditModal && (
        <Modal
          isOpen={!!showEditModal}
          onClose={() => setShowEditModal(null)}
          title="Edit RSS Source"
        >
          <EditFeedModal
            feed={showEditModal}
            onSave={(updates) => {
              handleUpdateFeed(showEditModal, updates);
            }}
            onClose={() => setShowEditModal(null)}
          />
        </Modal>
      )}
    </div>
  );
}

function EditFeedModal({ feed, onSave, onClose }: { 
  feed: RSSFeed; 
  onSave: (updates: Partial<RSSFeed>) => void;
  onClose: () => void;
}) {
  const [name, setName] = useState(feed.name);
  const [url, setUrl] = useState(feed.url);
  const [reliability, setReliability] = useState(feed.reliability);
  const [enabled, setEnabled] = useState(feed.enabled);

  const handleSave = () => {
    onSave({ name, url, reliability, enabled });
  };

  return (
    <div className="space-y-4">
      <div>
        <label className="block text-sm text-gray-400 mb-2">Source Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="glass-input w-full"
        />
      </div>
      <div>
        <label className="block text-sm text-gray-400 mb-2">RSS Feed URL</label>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          className="glass-input w-full"
        />
      </div>
      <div>
        <label className="block text-sm text-gray-400 mb-2">
          Reliability: {Math.round(reliability * 100)}%
        </label>
        <input
          type="range"
          min="0"
          max="1"
          step="0.1"
          value={reliability}
          onChange={(e) => setReliability(parseFloat(e.target.value))}
          className="w-full"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>Low</span>
          <span>High</span>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <input
          type="checkbox"
          id="enabled"
          checked={enabled}
          onChange={(e) => setEnabled(e.target.checked)}
          className="w-4 h-4"
        />
        <label htmlFor="enabled" className="text-sm text-gray-400">
          Enabled
        </label>
      </div>
      <div className="flex gap-2 justify-end">
        <Button variant="secondary" onClick={onClose}>
          Cancel
        </Button>
        <Button variant="primary" onClick={handleSave}>
          Save Changes
        </Button>
      </div>
    </div>
  );
}
