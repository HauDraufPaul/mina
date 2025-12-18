import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Rss, Plus, Link, Users, Network } from "lucide-react";

interface RSSFeed {
  id: number;
  url: string;
  name: string;
  enabled: boolean;
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

interface Entity {
  id: number;
  entity_type: string;
  name: string;
  metadata: string;
  created_at: number;
}

export default function RealityTimelineStudio() {
  const [feeds, setFeeds] = useState<RSSFeed[]>([]);
  const [items, setItems] = useState<RSSItem[]>([]);
  const [entities, setEntities] = useState<Entity[]>([]);
  const [view, setView] = useState<"feeds" | "items" | "entities">("feeds");
  const [loading, setLoading] = useState(true);
  const [newFeedUrl, setNewFeedUrl] = useState("");
  const [newFeedName, setNewFeedName] = useState("");

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    if (view === "items") {
      loadItems();
    } else if (view === "entities") {
      loadEntities();
    }
  }, [view]);

  const loadData = async () => {
    try {
      const feedsData = await invoke<RSSFeed[]>("list_rss_feeds");
      setFeeds(feedsData);
      setLoading(false);
    } catch (error) {
      console.error("Failed to load data:", error);
      setLoading(false);
    }
  };

  const loadItems = async () => {
    try {
      const itemsData = await invoke<RSSItem[]>("get_recent_rss_items", { limit: 50 });
      setItems(itemsData);
    } catch (error) {
      console.error("Failed to load items:", error);
    }
  };

  const loadEntities = async () => {
    try {
      const entitiesData = await invoke<Entity[]>("list_entities", { entityType: null });
      setEntities(entitiesData);
    } catch (error) {
      console.error("Failed to load entities:", error);
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
      });
      setNewFeedUrl("");
      setNewFeedName("");
      await loadData();
    } catch (error) {
      alert(`Failed to create feed: ${error}`);
    }
  };

  const handleCreateEntity = async () => {
    const entityType = prompt("Entity type (person/organization/location/event):");
    if (!entityType) return;
    const name = prompt("Entity name:");
    if (!name) return;
    const metadata = prompt("Metadata (JSON):") || "{}";

    try {
      await invoke("create_entity", {
        entityType,
        name,
        metadata,
      });
      await loadEntities();
    } catch (error) {
      alert(`Failed to create entity: ${error}`);
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  if (loading) {
    return <div className="text-center">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Reality & Timeline Studio
          </h1>
          <p className="text-gray-400">OSINT and entity extraction</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant={view === "feeds" ? "primary" : "secondary"}
            onClick={() => setView("feeds")}
          >
            <Rss className="w-4 h-4 mr-2" />
            RSS Feeds
          </Button>
          <Button
            variant={view === "items" ? "primary" : "secondary"}
            onClick={() => setView("items")}
          >
            <Link className="w-4 h-4 mr-2" />
            Feed Items
          </Button>
          <Button
            variant={view === "entities" ? "primary" : "secondary"}
            onClick={() => setView("entities")}
          >
            <Users className="w-4 h-4 mr-2" />
            Entities
          </Button>
        </div>
      </div>

      {view === "feeds" && (
        <>
          <Card title="Add RSS Feed">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Feed Name</label>
                <input
                  type="text"
                  value={newFeedName}
                  onChange={(e) => setNewFeedName(e.target.value)}
                  className="glass-input w-full"
                  placeholder="Feed Name"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">Feed URL</label>
                <input
                  type="text"
                  value={newFeedUrl}
                  onChange={(e) => setNewFeedUrl(e.target.value)}
                  className="glass-input w-full"
                  placeholder="https://example.com/feed.xml"
                />
              </div>
            </div>
            <Button variant="primary" className="mt-4" onClick={handleCreateFeed}>
              <Plus className="w-4 h-4 mr-2" />
              Add Feed
            </Button>
          </Card>

          <Card title="RSS Feeds">
            <div className="space-y-3">
              {feeds.length === 0 ? (
                <div className="text-center text-gray-400 py-8">
                  <Rss className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                  <p>No RSS feeds configured</p>
                </div>
              ) : (
                feeds.map((feed) => (
                  <div key={feed.id} className="glass-card p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="font-semibold">{feed.name}</div>
                        <div className="text-sm text-gray-400">{feed.url}</div>
                        <div className="text-xs text-gray-500 mt-1">
                          Created: {formatTime(feed.created_at)}
                          {feed.last_fetch && (
                            <span> • Last fetch: {formatTime(feed.last_fetch)}</span>
                          )}
                        </div>
                      </div>
                      <span
                        className={`text-xs px-2 py-1 rounded ${
                          feed.enabled
                            ? "bg-neon-green/20 text-neon-green"
                            : "bg-gray-500/20 text-gray-500"
                        }`}
                      >
                        {feed.enabled ? "Enabled" : "Disabled"}
                      </span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </Card>
        </>
      )}

      {view === "items" && (
        <Card title="Recent Feed Items">
          <div className="space-y-3">
            {items.length === 0 ? (
              <div className="text-center text-gray-400 py-8">
                <Link className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>No feed items available</p>
              </div>
            ) : (
              items.map((item) => (
                <div key={item.id} className="glass-card p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="font-semibold mb-2">{item.title}</div>
                      <p className="text-sm text-gray-300 mb-2 line-clamp-2">{item.content}</p>
                      <div className="flex items-center gap-4 text-xs text-gray-500">
                        <span>Published: {formatTime(item.published_at)}</span>
                        <a
                          href={item.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-neon-cyan hover:underline flex items-center gap-1"
                        >
                          <Link className="w-3 h-3" />
                          Open Link
                        </a>
                      </div>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </Card>
      )}

      {view === "entities" && (
        <>
          <Card title="Create Entity">
            <Button variant="primary" onClick={handleCreateEntity}>
              <Plus className="w-4 h-4 mr-2" />
              Create Entity
            </Button>
          </Card>

          <Card title="Entities">
            <div className="space-y-3">
              {entities.length === 0 ? (
                <div className="text-center text-gray-400 py-8">
                  <Users className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                  <p>No entities created yet</p>
                </div>
              ) : (
                entities.map((entity) => (
                  <div key={entity.id} className="glass-card p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="flex items-center gap-2">
                          <Network className="w-4 h-4 text-neon-cyan" />
                          <span className="font-semibold">{entity.name}</span>
                          <span className="text-xs text-gray-400">({entity.entity_type})</span>
                        </div>
                        <div className="text-xs text-gray-500 mt-1">
                          Created: {formatTime(entity.created_at)}
                        </div>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
