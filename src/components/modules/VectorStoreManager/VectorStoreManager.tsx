import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Database, Plus, Trash2, RefreshCw } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface CollectionStats {
  total: number;
  expired: number;
}

export default function VectorStoreManager() {
  const errorHandler = useErrorHandler();
  const [collections, setCollections] = useState<string[]>([]);
  const [selectedCollection, setSelectedCollection] = useState<string | null>(null);
  const [stats, setStats] = useState<CollectionStats | null>(null);
  const [newCollectionName, setNewCollectionName] = useState("");
  const [newCollectionDim, setNewCollectionDim] = useState(384);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadCollections();
  }, []);

  useEffect(() => {
    if (selectedCollection) {
      loadStats(selectedCollection);
    }
  }, [selectedCollection]);

  const loadCollections = async () => {
    try {
      const cols = await invoke<string[]>("list_collections");
      setCollections(cols);
      setLoading(false);
    } catch (error) {
      console.error("Failed to load collections:", error);
      setLoading(false);
    }
  };

  const loadStats = async (collection: string) => {
    try {
      const collectionStats = await invoke<CollectionStats>("get_collection_stats", {
        collection,
      });
      setStats(collectionStats);
    } catch (error) {
      console.error("Failed to load stats:", error);
    }
  };

  const handleCreateCollection = async () => {
    if (!newCollectionName.trim()) {
      errorHandler.showError("Collection name cannot be empty");
      return;
    }

    try {
      await invoke("create_collection", {
        name: newCollectionName,
        dimension: newCollectionDim,
      });
      setNewCollectionName("");
      setNewCollectionDim(384);
      await loadCollections();
      errorHandler.showSuccess("Collection created successfully");
    } catch (error) {
      errorHandler.showError("Failed to create collection", error);
    }
  };

  const handleCleanup = async () => {
    try {
      const count = await invoke<number>("cleanup_expired_vectors");
      errorHandler.showSuccess(`Cleaned up ${count} expired vectors`);
      if (selectedCollection) {
        await loadStats(selectedCollection);
      }
    } catch (error) {
      errorHandler.showError("Failed to cleanup", error);
    }
  };

  if (loading) {
    return <div className="text-center">Loading collections...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Vector Store Manager
          </h1>
          <p className="text-gray-400">Vector embeddings and semantic search</p>
        </div>
        <Button onClick={loadCollections} variant="secondary">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <Card title="Collections" subtitle="Total collections">
          <div className="text-3xl font-bold text-neon-cyan">{collections.length}</div>
        </Card>
        <Card title="Total Vectors" subtitle="All documents">
          <div className="text-3xl font-bold text-neon-green">
            {stats?.total || 0}
          </div>
        </Card>
        <Card title="Expired" subtitle="TTL expired">
          <div className="text-3xl font-bold text-neon-amber">
            {stats?.expired || 0}
          </div>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Collections">
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {collections.length === 0 ? (
              <div className="text-center text-gray-400 py-8">
                <Database className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>No collections found</p>
              </div>
            ) : (
              collections.map((collection) => (
                <div
                  key={collection}
                  className={`glass-card p-3 flex items-center justify-between cursor-pointer transition-all ${
                    selectedCollection === collection
                      ? "border-2 border-neon-cyan"
                      : "hover:border border-white/10"
                  }`}
                  onClick={() => setSelectedCollection(collection)}
                >
                  <div className="flex items-center gap-2">
                    <Database className="w-4 h-4 text-neon-cyan" />
                    <span className="font-semibold font-mono">{collection}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </Card>

        <Card title={selectedCollection ? `Collection: ${selectedCollection}` : "Select a Collection"}>
          {selectedCollection && stats ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-sm text-gray-400 mb-1">Total Vectors</div>
                  <div className="text-2xl font-bold text-neon-green">{stats.total}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1">Expired</div>
                  <div className="text-2xl font-bold text-neon-amber">{stats.expired}</div>
                </div>
              </div>
              <div className="w-full bg-gray-800 rounded-full h-2">
                <div
                  className="bg-neon-cyan h-2 rounded-full"
                  style={{
                    width: `${stats.total > 0 ? ((stats.total - stats.expired) / stats.total) * 100 : 0}%`,
                  }}
                />
              </div>
              <Button onClick={handleCleanup} variant="secondary" className="w-full">
                <Trash2 className="w-4 h-4 mr-2" />
                Cleanup Expired
              </Button>
            </div>
          ) : (
            <div className="text-center text-gray-400 py-8">
              Select a collection to view details
            </div>
          )}
        </Card>
      </div>

      <Card title="Create New Collection">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Collection Name</label>
            <input
              type="text"
              value={newCollectionName}
              onChange={(e) => setNewCollectionName(e.target.value)}
              className="glass-input w-full"
              placeholder="my_collection"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">
              Vector Dimension: {newCollectionDim}
            </label>
            <input
              type="range"
              min="128"
              max="2048"
              step="128"
              value={newCollectionDim}
              onChange={(e) => setNewCollectionDim(parseInt(e.target.value))}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>128</span>
              <span>2048</span>
            </div>
          </div>
          <Button onClick={handleCreateCollection} variant="primary" className="w-full">
            <Plus className="w-4 h-4 mr-2" />
            Create Collection
          </Button>
        </div>
      </Card>

      <Card title="Vector Store Information">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <div className="text-sm text-gray-400 mb-1">Storage Backend</div>
            <div className="font-mono text-sm">SQLite (Embedded)</div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">Similarity Metric</div>
            <div className="font-mono text-sm">Cosine Similarity</div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">TTL Support</div>
            <div className="font-mono text-sm">Enabled</div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">Index Type</div>
            <div className="font-mono text-sm">Linear Search</div>
          </div>
        </div>
      </Card>
    </div>
  );
}
