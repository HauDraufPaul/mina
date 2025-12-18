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
  RefreshCw,
  BookOpen,
  Bookmark,
  Folder,
  Printer,
  Download,
  EyeOff,
  Tag,
  Users,
  Building2,
  MapPin,
  Link as LinkIcon,
  Circle,
  Settings,
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
  read: boolean;
  favorite: boolean;
  saved: boolean;
  folder_id?: number;
}

interface ArticleFolder {
  id: number;
  name: string;
  color?: string;
  created_at: number;
}

interface ExtractedEntity {
  id: number;
  article_id: number;
  entity_type: string;
  name: string;
  confidence: number;
  context?: string;
  extracted_at: number;
}

type Tab = "sources" | "reader";

export default function RealityTimelineStudio() {
  const [activeTab, setActiveTab] = useState<Tab>("sources");
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

  // RSS Reader state
  const [selectedArticle, setSelectedArticle] = useState<RSSItem | null>(null);
  const [entities, setEntities] = useState<ExtractedEntity[]>([]);
  const [folders, setFolders] = useState<ArticleFolder[]>([]);
  const [loadingEntities, setLoadingEntities] = useState(false);
  const [extractingEntities, setExtractingEntities] = useState(false);
  const [filterFavorite, setFilterFavorite] = useState<boolean | null>(null);
  const [filterSaved, setFilterSaved] = useState<boolean | null>(null);
  const [filterRead, setFilterRead] = useState<boolean | null>(null);
  const [filterFolder, setFilterFolder] = useState<number | null>(null);
  const [showFolderModal, setShowFolderModal] = useState(false);
  const [newFolderName, setNewFolderName] = useState("");
  const [newFolderColor, setNewFolderColor] = useState("#3b82f6");
  const articleContentRef = useRef<HTMLDivElement>(null);

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
    if (activeTab === "reader") {
      loadFolders();
    }
  }, [activeTab]);

  useEffect(() => {
    if (selectedArticle) {
      loadEntities(selectedArticle.id);
      markAsRead(selectedArticle.id);
    }
  }, [selectedArticle]);

  useEffect(() => {
    if (activeTab === "reader") {
      loadReaderArticles();
    }
  }, [filterFavorite, filterSaved, filterRead, filterFolder, activeTab]);

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

  const loadReaderArticles = async () => {
    try {
      const items = await invoke<RSSItem[]>("get_filtered_articles", {
        favorite: filterFavorite,
        saved: filterSaved,
        read: filterRead,
        folder_id: filterFolder,
        limit: 500,
      });
      setItems(items);
    } catch (error) {
      console.error("Failed to load articles:", error);
    }
  };

  const loadFolders = async () => {
    try {
      const foldersData = await invoke<ArticleFolder[]>("list_article_folders");
      setFolders(foldersData);
    } catch (error) {
      console.error("Failed to load folders:", error);
    }
  };

  const loadEntities = async (articleId: number) => {
    try {
      setLoadingEntities(true);
      const entitiesData = await invoke<ExtractedEntity[]>("get_article_entities", {
        article_id: articleId,
      });
      setEntities(entitiesData);
    } catch (error) {
      console.error("Failed to load entities:", error);
    } finally {
      setLoadingEntities(false);
    }
  };

  const markAsRead = async (id: number) => {
    try {
      await invoke("mark_article_read", { id, read: true });
      setItems(prev => prev.map(a => a.id === id ? { ...a, read: true } : a));
      if (selectedArticle?.id === id) {
        setSelectedArticle(prev => prev ? { ...prev, read: true } : null);
      }
    } catch (error) {
      console.error("Failed to mark as read:", error);
    }
  };

  const toggleFavorite = async (id: number) => {
    try {
      const isFavorite = await invoke<boolean>("toggle_article_favorite", { id });
      setItems(prev => prev.map(a => a.id === id ? { ...a, favorite: isFavorite } : a));
      if (selectedArticle?.id === id) {
        setSelectedArticle(prev => prev ? { ...prev, favorite: isFavorite } : null);
      }
    } catch (error) {
      console.error("Failed to toggle favorite:", error);
    }
  };

  const toggleSaved = async (id: number) => {
    try {
      const isSaved = await invoke<boolean>("toggle_article_saved", { id });
      setItems(prev => prev.map(a => a.id === id ? { ...a, saved: isSaved } : a));
      if (selectedArticle?.id === id) {
        setSelectedArticle(prev => prev ? { ...prev, saved: isSaved } : null);
      }
    } catch (error) {
      console.error("Failed to toggle saved:", error);
    }
  };

  const setFolder = async (id: number, folderId: number | null) => {
    try {
      await invoke("set_article_folder", { id, folder_id: folderId });
      await loadReaderArticles();
      if (selectedArticle?.id === id) {
        const updated = await invoke<RSSItem | null>("get_rss_item", { id });
        if (updated) setSelectedArticle(updated);
      }
    } catch (error) {
      console.error("Failed to set folder:", error);
    }
  };

  const createFolder = async () => {
    if (!newFolderName.trim()) return;
    try {
      await invoke("create_article_folder", {
        name: newFolderName,
        color: newFolderColor,
      });
      setNewFolderName("");
      setNewFolderColor("#3b82f6");
      setShowFolderModal(false);
      await loadFolders();
    } catch (error) {
      console.error("Failed to create folder:", error);
      alert(`Failed to create folder: ${error}`);
    }
  };

  const deleteFolder = async (id: number) => {
    if (!confirm("Delete this folder? Articles will be moved to uncategorized.")) return;
    try {
      await invoke("delete_article_folder", { id });
      await loadFolders();
      await loadReaderArticles();
    } catch (error) {
      console.error("Failed to delete folder:", error);
    }
  };

  const extractEntities = async () => {
    if (!selectedArticle) return;
    try {
      setExtractingEntities(true);
      const count = await invoke<number>("extract_entities_from_article", {
        article_id: selectedArticle.id,
      });
      await loadEntities(selectedArticle.id);
      alert(`Extracted ${count} entities from article!`);
    } catch (error) {
      console.error("Failed to extract entities:", error);
      alert(`Failed to extract entities: ${error}`);
    } finally {
      setExtractingEntities(false);
    }
  };

  const handlePrint = () => {
    if (!selectedArticle) return;
    window.print();
  };

  const handleSaveArticle = async () => {
    if (!selectedArticle) return;
    const content = `
Title: ${selectedArticle.title}
Source: ${feeds.find(f => f.id === selectedArticle.feed_id)?.name || "Unknown"}
Published: ${new Date(selectedArticle.published_at * 1000).toLocaleString()}
URL: ${selectedArticle.url}

${selectedArticle.content}
    `.trim();
    
    navigator.clipboard.writeText(content);
    alert("Article content copied to clipboard!");
  };

  const handleFetchArticles = async () => {
    try {
      setLoadingItems(true);
      const enabledFeeds = feeds.filter(f => f.enabled);
      if (enabledFeeds.length === 0) {
        alert("No enabled feeds to fetch from. Please enable at least one RSS feed.");
        setLoadingItems(false);
        return;
      }

      const count = await invoke<number>("fetch_rss_feeds");
      await loadItems();
      await loadReaderArticles();
      await loadFeeds();
      alert(`Successfully fetched ${count} article(s) from ${enabledFeeds.length} feed(s)!`);
    } catch (error) {
      console.error("Failed to fetch articles:", error);
      alert(`Failed to fetch articles: ${error}`);
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
      await loadReaderArticles();
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

    if (filterEnabled === "enabled") {
      filtered = filtered.filter(f => f.enabled);
    } else if (filterEnabled === "disabled") {
      filtered = filtered.filter(f => !f.enabled);
    }

    if (debouncedSearchQuery.trim()) {
      const query = debouncedSearchQuery.toLowerCase();
      filtered = filtered.filter(f => 
        f.name.toLowerCase().includes(query) ||
        f.url.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [feeds, debouncedSearchQuery, filterEnabled]);

  // Filtered items for sources view
  const filteredItems = useMemo(() => {
    let filtered = items;

    if (debouncedSearchQuery.trim()) {
      const query = debouncedSearchQuery.toLowerCase();
      filtered = filtered.filter(item => 
        item.title.toLowerCase().includes(query) ||
        item.content.toLowerCase().includes(query)
      );
    }

    return filtered;
  }, [items, debouncedSearchQuery]);

  // Filtered articles for reader view
  const filteredArticles = useMemo(() => {
    let filtered = items;
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(a =>
        a.title.toLowerCase().includes(query) ||
        a.content.toLowerCase().includes(query)
      );
    }
    return filtered;
  }, [items, searchQuery]);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    const minutes = Math.floor(diff / (1000 * 60));
    return minutes > 0 ? `${minutes}m ago` : "Just now";
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

  const getEntityIcon = (type: string) => {
    switch (type.toLowerCase()) {
      case "person": return Users;
      case "company": return Building2;
      case "country": case "location": return MapPin;
      case "connection": return LinkIcon;
      default: return Tag;
    }
  };

  const getEntityColor = (type: string) => {
    switch (type.toLowerCase()) {
      case "person": return "text-neon-cyan";
      case "company": return "text-neon-amber";
      case "country": case "location": return "text-neon-green";
      case "connection": return "text-purple-400";
      default: return "text-gray-400";
    }
  };

  const groupedEntities = useMemo(() => {
    const grouped: Record<string, ExtractedEntity[]> = {};
    entities.forEach(entity => {
      if (!grouped[entity.entity_type]) {
        grouped[entity.entity_type] = [];
      }
      grouped[entity.entity_type].push(entity);
    });
    return grouped;
  }, [entities]);

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
          <p className="text-gray-400">OSINT Intelligence Hub & RSS Reader</p>
        </div>
        <div className="flex items-center gap-2">
          {activeTab === "sources" && (
            <Button variant="primary" onClick={() => setShowAddModal(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Add RSS Source
            </Button>
          )}
          <Button variant="secondary" onClick={handleFetchArticles} disabled={loadingItems}>
            <RefreshCw className={`w-4 h-4 mr-2 ${loadingItems ? "animate-spin" : ""}`} />
            Fetch Articles
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setActiveTab("sources")}
          className={`px-6 py-3 rounded-lg font-semibold transition-all ${
            activeTab === "sources"
              ? "bg-neon-cyan/20 text-neon-cyan border border-neon-cyan"
              : "bg-white/5 text-gray-400 hover:bg-white/10"
          }`}
        >
          <Settings className="w-4 h-4 inline mr-2" />
          Sources
        </button>
        <button
          onClick={() => setActiveTab("reader")}
          className={`px-6 py-3 rounded-lg font-semibold transition-all ${
            activeTab === "reader"
              ? "bg-neon-cyan/20 text-neon-cyan border border-neon-cyan"
              : "bg-white/5 text-gray-400 hover:bg-white/10"
          }`}
        >
          <BookOpen className="w-4 h-4 inline mr-2" />
          Reader
        </button>
      </div>

      {/* Sources Tab */}
      {activeTab === "sources" && (
        <>
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
                <p className="text-sm mb-4">Click "Fetch Articles" to retrieve articles from your enabled RSS feeds</p>
                <Button variant="primary" onClick={handleFetchArticles}>
                  <RefreshCw className="w-4 h-4 mr-2" />
                  Fetch Articles Now
                </Button>
              </div>
            ) : (
              <div className="space-y-4">
                {filteredItems.map((item) => {
                  const feed = feeds.find(f => f.id === item.feed_id);
                  return (
                    <div
                      key={item.id}
                      className="glass-card p-4 hover:border-neon-cyan/50 transition-all group cursor-pointer"
                      onClick={() => {
                        setActiveTab("reader");
                        setSelectedArticle(item);
                      }}
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
                            onClick={(e) => e.stopPropagation()}
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
        </>
      )}

      {/* Reader Tab */}
      {activeTab === "reader" && (
        <div className="flex gap-6 h-[calc(100vh-300px)]">
          {/* Sidebar - Article List */}
          <div className="w-96 flex flex-col">
            {/* Filters */}
            <Card className="mb-4">
              <div className="space-y-4">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                  <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search articles..."
                    className="glass-input w-full pl-10"
                  />
                </div>

                <div className="flex flex-wrap gap-2">
                  <button
                    onClick={() => setFilterFavorite(filterFavorite === true ? null : true)}
                    className={`px-3 py-1 rounded text-sm transition-colors ${
                      filterFavorite === true
                        ? "bg-neon-amber/20 text-neon-amber border border-neon-amber"
                        : "bg-white/5 text-gray-400 hover:bg-white/10"
                    }`}
                  >
                    <Star className="w-3 h-3 inline mr-1" />
                    Favorites
                  </button>
                  <button
                    onClick={() => setFilterSaved(filterSaved === true ? null : true)}
                    className={`px-3 py-1 rounded text-sm transition-colors ${
                      filterSaved === true
                        ? "bg-neon-green/20 text-neon-green border border-neon-green"
                        : "bg-white/5 text-gray-400 hover:bg-white/10"
                    }`}
                  >
                    <Bookmark className="w-3 h-3 inline mr-1" />
                    Saved
                  </button>
                  <button
                    onClick={() => setFilterRead(filterRead === false ? null : false)}
                    className={`px-3 py-1 rounded text-sm transition-colors ${
                      filterRead === false
                        ? "bg-neon-cyan/20 text-neon-cyan border border-neon-cyan"
                        : "bg-white/5 text-gray-400 hover:bg-white/10"
                    }`}
                  >
                    <EyeOff className="w-3 h-3 inline mr-1" />
                    Unread
                  </button>
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm text-gray-400">Folders</span>
                    <button
                      onClick={() => setShowFolderModal(true)}
                      className="text-neon-cyan hover:text-neon-cyan/80"
                    >
                      <Plus className="w-4 h-4" />
                    </button>
                  </div>
                  <div className="space-y-1">
                    <button
                      onClick={() => setFilterFolder(null)}
                      className={`w-full text-left px-2 py-1 rounded text-sm ${
                        filterFolder === null
                          ? "bg-neon-cyan/20 text-neon-cyan"
                          : "text-gray-400 hover:bg-white/5"
                      }`}
                    >
                      All Articles
                    </button>
                    {folders.map(folder => (
                      <div key={folder.id} className="flex items-center gap-2">
                        <button
                          onClick={() => setFilterFolder(filterFolder === folder.id ? null : folder.id)}
                          className={`flex-1 text-left px-2 py-1 rounded text-sm ${
                            filterFolder === folder.id
                              ? "bg-neon-cyan/20 text-neon-cyan"
                              : "text-gray-400 hover:bg-white/5"
                          }`}
                        >
                          <Folder className="w-3 h-3 inline mr-1" />
                          {folder.name}
                        </button>
                        <button
                          onClick={() => deleteFolder(folder.id)}
                          className="text-red-500 hover:text-red-400"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </Card>

            {/* Article List */}
            <Card className="flex-1 overflow-y-auto">
              <div className="space-y-2">
                {filteredArticles.length === 0 ? (
                  <div className="text-center text-gray-400 py-8">
                    <BookOpen className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>No articles found</p>
                  </div>
                ) : (
                  filteredArticles.map(article => {
                    const feed = feeds.find(f => f.id === article.feed_id);
                    return (
                      <div
                        key={article.id}
                        onClick={() => setSelectedArticle(article)}
                        className={`glass-card p-3 cursor-pointer transition-all hover:border-neon-cyan/50 ${
                          selectedArticle?.id === article.id
                            ? "border-neon-cyan border-2"
                            : article.read
                            ? "opacity-60"
                            : ""
                        }`}
                      >
                        <div className="flex items-start justify-between gap-2">
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-1">
                              {!article.read && (
                                <Circle className="w-2 h-2 text-neon-cyan flex-shrink-0" />
                              )}
                              <h3 className="font-semibold text-sm truncate">{article.title}</h3>
                            </div>
                            {feed && (
                              <div className="text-xs text-gray-400 mb-1">{feed.name}</div>
                            )}
                            <div className="text-xs text-gray-500">{formatTime(article.published_at)}</div>
                          </div>
                          <div className="flex flex-col gap-1">
                            {article.favorite && (
                              <Star className="w-3 h-3 text-neon-amber fill-neon-amber" />
                            )}
                            {article.saved && (
                              <Bookmark className="w-3 h-3 text-neon-green fill-neon-green" />
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })
                )}
              </div>
            </Card>
          </div>

          {/* Main Content - Article Reader */}
          <div className="flex-1 flex flex-col">
            {selectedArticle ? (
              <>
                {/* Article Header */}
                <Card className="mb-4">
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <h1 className="text-2xl font-bold mb-2">{selectedArticle.title}</h1>
                      <div className="flex items-center gap-4 text-sm text-gray-400">
                        <span>
                          {feeds.find(f => f.id === selectedArticle.feed_id)?.name || "Unknown"}
                        </span>
                        <span>•</span>
                        <span>{formatTime(selectedArticle.published_at)}</span>
                        <a
                          href={selectedArticle.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-neon-cyan hover:text-neon-cyan/80 flex items-center gap-1"
                        >
                          <ExternalLink className="w-3 h-3" />
                          Open Original
                        </a>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => toggleFavorite(selectedArticle.id)}
                        className={`p-2 rounded transition-colors ${
                          selectedArticle.favorite
                            ? "bg-neon-amber/20 text-neon-amber"
                            : "bg-white/5 text-gray-400 hover:bg-white/10"
                        }`}
                        title="Favorite"
                      >
                        <Star className={`w-4 h-4 ${selectedArticle.favorite ? "fill-current" : ""}`} />
                      </button>
                      <button
                        onClick={() => toggleSaved(selectedArticle.id)}
                        className={`p-2 rounded transition-colors ${
                          selectedArticle.saved
                            ? "bg-neon-green/20 text-neon-green"
                            : "bg-white/5 text-gray-400 hover:bg-white/10"
                        }`}
                        title="Save"
                      >
                        <Bookmark className={`w-4 h-4 ${selectedArticle.saved ? "fill-current" : ""}`} />
                      </button>
                      <button
                        onClick={handleSaveArticle}
                        className="p-2 rounded bg-white/5 text-gray-400 hover:bg-white/10 transition-colors"
                        title="Save to File"
                      >
                        <Download className="w-4 h-4" />
                      </button>
                      <button
                        onClick={handlePrint}
                        className="p-2 rounded bg-white/5 text-gray-400 hover:bg-white/10 transition-colors"
                        title="Print"
                      >
                        <Printer className="w-4 h-4" />
                      </button>
                    </div>
                  </div>

                  {/* Folder Selection */}
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="text-sm text-gray-400">Folder:</span>
                    <select
                      value={selectedArticle.folder_id || ""}
                      onChange={(e) => setFolder(selectedArticle.id, e.target.value ? parseInt(e.target.value) : null)}
                      className="glass-input text-sm"
                    >
                      <option value="">Uncategorized</option>
                      {folders.map(folder => (
                        <option key={folder.id} value={folder.id}>
                          {folder.name}
                        </option>
                      ))}
                    </select>
                  </div>
                </Card>

                {/* Article Content */}
                <Card className="flex-1 overflow-y-auto mb-4 p-6">
                  {selectedArticle.content && selectedArticle.content.trim().length > 0 ? (
                    <>
                      {selectedArticle.content.includes("read-full-article") && (
                        <div className="mb-4 p-3 bg-neon-amber/10 border border-neon-amber/30 rounded-lg">
                          <div className="flex items-center justify-between">
                            <p className="text-sm text-gray-300">
                              This article only has a summary. Click below to fetch the full content.
                            </p>
                            <Button
                              variant="secondary"
                              onClick={async () => {
                                try {
                                  await invoke("fetch_full_article", {
                                    article_id: selectedArticle.id,
                                    url: selectedArticle.url,
                                  });
                                  // Reload the article
                                  const updated = await invoke<RSSItem | null>("get_rss_item", { id: selectedArticle.id });
                                  if (updated) {
                                    setSelectedArticle(updated);
                                    alert("Full article content fetched successfully!");
                                  }
                                } catch (error) {
                                  alert(`Failed to fetch full article: ${error}`);
                                }
                              }}
                            >
                              <RefreshCw className="w-4 h-4 mr-2" />
                              Fetch Full Article
                            </Button>
                          </div>
                        </div>
                      )}
                      <div
                        ref={articleContentRef}
                        className="article-content prose prose-invert max-w-none"
                        dangerouslySetInnerHTML={{ __html: selectedArticle.content }}
                      />
                    </>
                  ) : (
                    <div className="text-center py-12">
                      <p className="text-gray-400 mb-4">No content available for this article.</p>
                      <Button
                        variant="primary"
                        onClick={async () => {
                          try {
                            await invoke("fetch_full_article", {
                              article_id: selectedArticle.id,
                              url: selectedArticle.url,
                            });
                            const updated = await invoke<RSSItem | null>("get_rss_item", { id: selectedArticle.id });
                            if (updated) {
                              setSelectedArticle(updated);
                              alert("Full article content fetched successfully!");
                            }
                          } catch (error) {
                            alert(`Failed to fetch full article: ${error}`);
                          }
                        }}
                      >
                        <RefreshCw className="w-4 h-4 mr-2" />
                        Fetch Full Article
                      </Button>
                    </div>
                  )}
                  <style>{`
                    .article-content {
                      color: #e5e7eb;
                      line-height: 1.7;
                      font-size: 16px;
                      padding: 1rem 0;
                    }
                    .article-content > *:first-child {
                      margin-top: 0;
                    }
                    .article-content > *:last-child {
                      margin-bottom: 0;
                    }
                    .article-content h1, .article-content h2, .article-content h3, .article-content h4 {
                      color: #ffffff;
                      font-weight: 700;
                      margin-top: 2em;
                      margin-bottom: 1em;
                    }
                    .article-content h1 { font-size: 2em; }
                    .article-content h2 { font-size: 1.75em; }
                    .article-content h3 { font-size: 1.5em; }
                    .article-content h4 { font-size: 1.25em; }
                    .article-content p {
                      color: #d1d5db;
                      margin-bottom: 1.5em;
                      line-height: 1.8;
                    }
                    .article-content a {
                      color: #06b6d4;
                      text-decoration: underline;
                      transition: color 0.2s;
                    }
                    .article-content a:hover {
                      color: #22d3ee;
                    }
                    .article-content .read-full-article {
                      display: inline-flex;
                      align-items: center;
                      gap: 0.5rem;
                      padding: 0.75rem 1.5rem;
                      background: rgba(6, 182, 212, 0.2);
                      border: 1px solid rgba(6, 182, 212, 0.5);
                      border-radius: 6px;
                      margin-top: 2rem;
                      font-weight: 600;
                      transition: all 0.2s;
                    }
                    .article-content .read-full-article:hover {
                      background: rgba(6, 182, 212, 0.3);
                      border-color: rgba(6, 182, 212, 0.8);
                      transform: translateX(4px);
                    }
                    .article-content .read-more {
                      margin-top: 2rem;
                      padding-top: 1rem;
                      border-top: 1px solid rgba(255, 255, 255, 0.1);
                    }
                    .article-content .no-content {
                      color: #9ca3af;
                      font-style: italic;
                    }
                    .article-content img {
                      max-width: 100%;
                      height: auto;
                      border-radius: 8px;
                      margin: 2em 0;
                      box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
                      display: block;
                      margin-left: auto;
                      margin-right: auto;
                    }
                    .article-content figure {
                      margin: 2em 0;
                      text-align: center;
                    }
                    .article-content figure img {
                      margin: 0;
                    }
                    .article-content figcaption {
                      color: #9ca3af;
                      font-size: 0.875em;
                      margin-top: 0.5em;
                      font-style: italic;
                    }
                    .article-content ul, .article-content ol {
                      color: #d1d5db;
                      margin: 1.5em 0;
                      padding-left: 2em;
                    }
                    .article-content li {
                      margin: 0.5em 0;
                      line-height: 1.8;
                    }
                    .article-content blockquote {
                      border-left: 4px solid #06b6d4;
                      padding-left: 1.5em;
                      margin: 2em 0;
                      color: #9ca3af;
                      font-style: italic;
                      background: rgba(6, 182, 212, 0.1);
                      padding: 1em 1.5em;
                      border-radius: 4px;
                    }
                    .article-content code {
                      background: rgba(251, 191, 36, 0.2);
                      color: #fbbf24;
                      padding: 0.2em 0.4em;
                      border-radius: 4px;
                      font-size: 0.9em;
                    }
                    .article-content pre {
                      background: rgba(0, 0, 0, 0.5);
                      padding: 1.5em;
                      border-radius: 8px;
                      overflow-x: auto;
                      margin: 2em 0;
                      border: 1px solid rgba(255, 255, 255, 0.1);
                    }
                    .article-content pre code {
                      background: transparent;
                      color: #fbbf24;
                      padding: 0;
                    }
                    .article-content strong, .article-content b {
                      color: #ffffff;
                      font-weight: 700;
                    }
                    .article-content em, .article-content i {
                      font-style: italic;
                    }
                    .article-content table {
                      width: 100%;
                      border-collapse: collapse;
                      margin: 2em 0;
                    }
                    .article-content table th,
                    .article-content table td {
                      border: 1px solid rgba(255, 255, 255, 0.1);
                      padding: 0.75em;
                      text-align: left;
                    }
                    .article-content table th {
                      background: rgba(6, 182, 212, 0.2);
                      color: #ffffff;
                      font-weight: 700;
                    }
                    .article-content table td {
                      color: #d1d5db;
                    }
                    .article-content hr {
                      border: none;
                      border-top: 1px solid rgba(255, 255, 255, 0.1);
                      margin: 3em 0;
                    }
                    .article-content iframe,
                    .article-content video,
                    .article-content embed {
                      max-width: 100%;
                      border-radius: 8px;
                      margin: 2em 0;
                    }
                  `}</style>
                </Card>

                {/* Extracted Entities */}
                <Card title="Extracted Entities" className="max-h-64 overflow-y-auto">
                  <div className="mb-4">
                    <Button
                      variant="secondary"
                      onClick={extractEntities}
                      disabled={extractingEntities}
                    >
                      {extractingEntities ? (
                        <>
                          <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                          Extracting...
                        </>
                      ) : (
                        <>
                          <Tag className="w-4 h-4 mr-2" />
                          Extract Entities
                        </>
                      )}
                    </Button>
                  </div>

                  {loadingEntities ? (
                    <div className="flex items-center justify-center py-8">
                      <Loader2 className="w-6 h-6 text-neon-cyan animate-spin" />
                    </div>
                  ) : entities.length === 0 ? (
                    <div className="text-center text-gray-400 py-8">
                      <Tag className="w-12 h-12 mx-auto mb-4 opacity-50" />
                      <p>No entities extracted yet</p>
                      <p className="text-sm mt-2">Click "Extract Entities" to analyze this article</p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {Object.entries(groupedEntities).map(([type, typeEntities]) => {
                        const Icon = getEntityIcon(type);
                        const colorClass = getEntityColor(type);
                        return (
                          <div key={type}>
                            <div className="flex items-center gap-2 mb-2">
                              <Icon className={`w-4 h-4 ${colorClass}`} />
                              <h3 className="font-semibold capitalize">{type}</h3>
                              <span className="text-xs text-gray-500">({typeEntities.length})</span>
                            </div>
                            <div className="flex flex-wrap gap-2">
                              {typeEntities.map(entity => (
                                <div
                                  key={entity.id}
                                  className="glass-card px-3 py-1 text-sm flex items-center gap-2"
                                >
                                  <span>{entity.name}</span>
                                  <span className="text-xs text-gray-500">
                                    {Math.round(entity.confidence * 100)}%
                                  </span>
                                </div>
                              ))}
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </Card>
              </>
            ) : (
              <Card className="flex-1 flex items-center justify-center">
                <div className="text-center text-gray-400">
                  <BookOpen className="w-16 h-16 mx-auto mb-4 opacity-50" />
                  <p className="text-lg mb-2">Select an article to read</p>
                  <p className="text-sm">Choose an article from the list to view its content</p>
                </div>
              </Card>
            )}
          </div>
        </div>
      )}

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

      {/* Create Folder Modal */}
      <Modal
        isOpen={showFolderModal}
        onClose={() => {
          setShowFolderModal(false);
          setNewFolderName("");
          setNewFolderColor("#3b82f6");
        }}
        title="Create Folder"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Folder Name</label>
            <input
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              placeholder="e.g., Tech News"
              className="glass-input w-full"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Color</label>
            <input
              type="color"
              value={newFolderColor}
              onChange={(e) => setNewFolderColor(e.target.value)}
              className="w-full h-10 rounded"
            />
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowFolderModal(false);
                setNewFolderName("");
                setNewFolderColor("#3b82f6");
              }}
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={createFolder}>
              Create
            </Button>
          </div>
        </div>
      </Modal>
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
