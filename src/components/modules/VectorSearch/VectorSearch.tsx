import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Search, Filter, Database } from "lucide-react";

interface SearchResult {
  document: {
    id: string;
    collection: string;
    content: string;
    metadata: Record<string, string>;
  };
  similarity: number;
}

export default function VectorSearch() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [collections, setCollections] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [filters, setFilters] = useState({
    minSimilarity: 0.7,
    collection: "all",
    limit: 10,
  });

  useEffect(() => {
    loadCollections();
  }, []);

  const loadCollections = async () => {
    try {
      const cols = await invoke<string[]>("list_collections");
      setCollections(cols);
      if (cols.length > 0 && filters.collection === "all") {
        setFilters({ ...filters, collection: cols[0] });
      }
    } catch (error) {
      console.error("Failed to load collections:", error);
    }
  };

  const handleSearch = async () => {
    if (!query.trim()) {
      alert("Please enter a search query");
      return;
    }

    if (filters.collection === "all") {
      alert("Please select a collection");
      return;
    }

    setLoading(true);
    try {
      // For now, we'll use a simple text-based approach
      // In production, you'd generate embeddings from the query
      // For demonstration, we'll create a dummy embedding vector
      // The backend will handle the actual similarity search
      
      // Create a simple embedding vector (384 dimensions, all zeros for now)
      // In a real implementation, you'd use an embedding model here
      const queryEmbedding = new Array(384).fill(0).map(() => Math.random() * 0.1);

      const searchResults = await invoke<SearchResult[]>("search_vectors", {
        collection: filters.collection,
        queryEmbedding: queryEmbedding,
        limit: filters.limit,
        minSimilarity: filters.minSimilarity,
      });

      setResults(searchResults);
    } catch (error) {
      console.error("Search failed:", error);
      alert(`Search failed: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const getSimilarityColor = (similarity: number) => {
    if (similarity >= 0.9) return "text-neon-green";
    if (similarity >= 0.8) return "text-neon-cyan";
    if (similarity >= 0.7) return "text-neon-amber";
    return "text-gray-400";
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Vector Search
        </h1>
        <p className="text-gray-400">Semantic vector search and similarity matching</p>
      </div>

      <Card title="Search Query">
        <div className="space-y-4">
          <div className="flex gap-3">
            <div className="flex-1">
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyPress={(e) => e.key === "Enter" && handleSearch()}
                className="glass-input w-full"
                placeholder="Enter your search query..."
              />
            </div>
            <Button onClick={handleSearch} variant="primary" disabled={loading}>
              <Search className="w-4 h-4 mr-2" />
              Search
            </Button>
          </div>
        </div>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <Card title="Filters" className="lg:col-span-1">
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">
                Min Similarity: {filters.minSimilarity.toFixed(2)}
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={filters.minSimilarity}
                onChange={(e) =>
                  setFilters({ ...filters, minSimilarity: parseFloat(e.target.value) })
                }
                className="w-full"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Collection</label>
              <select
                value={filters.collection}
                onChange={(e) => setFilters({ ...filters, collection: e.target.value })}
                className="glass-input w-full"
              >
                <option value="all">All Collections</option>
                {collections.map((col) => (
                  <option key={col} value={col}>
                    {col}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Result Limit</label>
              <input
                type="number"
                min="1"
                max="100"
                value={filters.limit}
                onChange={(e) =>
                  setFilters({ ...filters, limit: parseInt(e.target.value) || 10 })
                }
                className="glass-input w-full"
              />
            </div>
          </div>
        </Card>

        <div className="lg:col-span-3 space-y-6">
          {loading ? (
            <Card title="Searching...">
              <div className="text-center py-8 text-gray-400">
                Performing semantic search...
              </div>
            </Card>
          ) : results.length === 0 ? (
            <Card title="No Results">
              <div className="text-center py-8 text-gray-400">
                <Search className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>Enter a query and click Search to find similar content</p>
                <p className="text-xs mt-2 text-gray-500">
                  Note: Embedding generation is simplified. In production, use a proper embedding model.
                </p>
              </div>
            </Card>
          ) : (
            <>
              <Card title={`Search Results (${results.length})`}>
                <div className="space-y-4">
                  {results.map((result, index) => (
                    <div
                      key={index}
                      className="glass-card p-4 hover:border-neon-cyan transition-all"
                    >
                      <div className="flex items-start justify-between mb-2">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <Database className="w-4 h-4 text-neon-cyan" />
                            <span className="text-xs text-gray-400 font-mono">
                              {result.document.collection}
                            </span>
                            {result.document.metadata && Object.keys(result.document.metadata).length > 0 && (
                              <span className="text-xs text-gray-500">
                                • {Object.keys(result.document.metadata)[0]}
                              </span>
                            )}
                          </div>
                          <p className="text-sm text-gray-300 mb-2">{result.document.content}</p>
                        </div>
                        <div
                          className={`ml-4 text-right ${getSimilarityColor(
                            result.similarity
                          )}`}
                        >
                          <div className="text-2xl font-bold">
                            {(result.similarity * 100).toFixed(0)}%
                          </div>
                          <div className="text-xs">similarity</div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2 text-xs text-gray-500">
                        <Filter className="w-3 h-3" />
                        <span>ID: {result.document.id}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </Card>

              <Card title="Search Analytics">
                <div className="grid grid-cols-3 gap-4">
                  <div className="text-center">
                    <div className="text-2xl font-bold text-neon-cyan mb-1">
                      {results.length}
                    </div>
                    <div className="text-xs text-gray-400">Results Found</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-neon-green mb-1">
                      {results.length > 0
                        ? (
                            results.reduce((sum, r) => sum + r.similarity, 0) /
                            results.length
                          ).toFixed(2)
                        : "0.00"}
                    </div>
                    <div className="text-xs text-gray-400">Avg Similarity</div>
                  </div>
                  <div className="text-center">
                    <div className="text-2xl font-bold text-neon-amber mb-1">
                      {results.length > 0
                        ? Math.max(...results.map((r) => r.similarity)).toFixed(2)
                        : "0.00"}
                    </div>
                    <div className="text-xs text-gray-400">Max Similarity</div>
                  </div>
                </div>
              </Card>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
