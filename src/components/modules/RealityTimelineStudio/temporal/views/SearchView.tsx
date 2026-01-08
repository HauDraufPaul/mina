import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Search } from "lucide-react";

interface SearchHit {
  doc_type: "rss_item" | "temporal_event";
  doc_id: number;
  title: string;
  snippet: string;
  ts: number;
}

export default function SearchView() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchHit[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runSearch = async () => {
    const q = query.trim();
    if (!q) return;
    setLoading(true);
    setError(null);
    try {
      const res = await invoke<SearchHit[]>("temporal_search", { query: q, limit: 100 });
      setResults(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Search failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <Card title="Search" subtitle="Full-text search across RSS items and temporal events (FTS5)">
        <div className="flex items-center gap-2">
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Try: NVIDIA OR 'export controls'"
            className="flex-1 bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-neon-cyan"
            onKeyDown={(e) => {
              if (e.key === "Enter") runSearch();
            }}
          />
          <Button onClick={runSearch} variant="primary" disabled={loading || !query.trim()}>
            <Search className="w-4 h-4 mr-2" />
            Search
          </Button>
        </div>
        {error && <div className="mt-3 text-neon-red text-sm">{error}</div>}
      </Card>

      <Card title="Results" subtitle={loading ? "Searching…" : `${results.length} hits`}>
        {loading ? (
          <div className="text-gray-400">Searching…</div>
        ) : results.length === 0 ? (
          <div className="text-gray-400">No results. You may need to “Rebuild Events” to index docs.</div>
        ) : (
          <div className="space-y-3">
            {results.map((hit) => (
              <div key={`${hit.doc_type}-${hit.doc_id}`} className="glass-card p-4">
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="text-xs text-gray-500">
                      {hit.doc_type} • id {hit.doc_id} • {new Date(hit.ts * 1000).toLocaleString()}
                    </div>
                    <div className="text-neon-cyan font-semibold mt-1 truncate">{hit.title}</div>
                    <div
                      className="text-sm text-gray-300 mt-2"
                      dangerouslySetInnerHTML={{ __html: hit.snippet }}
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}


