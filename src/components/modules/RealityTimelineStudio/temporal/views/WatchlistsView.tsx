import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Plus, List } from "lucide-react";

interface Watchlist {
  id: number;
  name: string;
  created_at: number;
}

interface WatchlistItem {
  id: number;
  watchlist_id: number;
  item_type: string;
  value: string;
  weight: number;
  enabled: boolean;
  created_at: number;
}

export default function WatchlistsView() {
  const [watchlists, setWatchlists] = useState<Watchlist[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [items, setItems] = useState<WatchlistItem[]>([]);
  const [newListName, setNewListName] = useState("");
  const [newItemType, setNewItemType] = useState("keyword");
  const [newItemValue, setNewItemValue] = useState("");
  const [loading, setLoading] = useState(true);

  const loadWatchlists = async () => {
    setLoading(true);
    try {
      const data = await invoke<Watchlist[]>("temporal_list_watchlists");
      setWatchlists(data);
      if (!selectedId && data.length > 0) {
        setSelectedId(data[0].id);
      }
    } finally {
      setLoading(false);
    }
  };

  const loadItems = async (id: number) => {
    const data = await invoke<WatchlistItem[]>("temporal_list_watchlist_items", { watchlistId: id });
    setItems(data);
  };

  useEffect(() => {
    loadWatchlists();
  }, []);

  useEffect(() => {
    if (selectedId) loadItems(selectedId);
  }, [selectedId]);

  const createWatchlist = async () => {
    const name = newListName.trim();
    if (!name) return;
    await invoke<number>("temporal_create_watchlist", { name });
    setNewListName("");
    await loadWatchlists();
  };

  const addItem = async () => {
    if (!selectedId) return;
    const value = newItemValue.trim();
    if (!value) return;
    await invoke<number>("temporal_add_watchlist_item", {
      watchlistId: selectedId,
      itemType: newItemType,
      value,
      weight: 1.0,
      enabled: true,
    });
    setNewItemValue("");
    await loadItems(selectedId);
  };

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card title="Watchlists" subtitle="Track entities/keywords/themes">
          {loading ? (
            <div className="text-gray-400">Loading…</div>
          ) : (
            <div className="space-y-2">
              {watchlists.map((wl) => (
                <button
                  key={wl.id}
                  onClick={() => setSelectedId(wl.id)}
                  className={`w-full text-left glass-card p-3 ${
                    selectedId === wl.id ? "border border-neon-cyan/40" : ""
                  }`}
                >
                  <div className="font-semibold text-gray-200">{wl.name}</div>
                  <div className="text-xs text-gray-500">#{wl.id}</div>
                </button>
              ))}
            </div>
          )}

          <div className="mt-4 pt-4 border-t border-white/10 space-y-2">
            <div className="text-xs text-gray-500">Create watchlist</div>
            <input
              value={newListName}
              onChange={(e) => setNewListName(e.target.value)}
              placeholder="e.g., Semiconductors"
              className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
            />
            <Button onClick={createWatchlist} variant="secondary" disabled={!newListName.trim()}>
              <Plus className="w-4 h-4 mr-2" />
              Create
            </Button>
          </div>
        </Card>

        <div className="md:col-span-2">
          <Card
            title="Watchlist Items"
            subtitle={selectedId ? `Watchlist #${selectedId} • ${items.length} items` : "Select a watchlist"}
          >
            {!selectedId ? (
              <div className="text-gray-400">Select a watchlist to view items.</div>
            ) : (
              <>
                <div className="flex flex-col md:flex-row gap-2 mb-4">
                  <select
                    value={newItemType}
                    onChange={(e) => setNewItemType(e.target.value)}
                    className="bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
                  >
                    <option value="keyword">keyword</option>
                    <option value="entity">entity</option>
                    <option value="domain">domain</option>
                    <option value="source">source</option>
                  </select>
                  <input
                    value={newItemValue}
                    onChange={(e) => setNewItemValue(e.target.value)}
                    placeholder="Value (e.g., NVIDIA, export controls)"
                    className="flex-1 bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
                    onKeyDown={(e) => {
                      if (e.key === "Enter") addItem();
                    }}
                  />
                  <Button onClick={addItem} variant="primary" disabled={!newItemValue.trim()}>
                    <Plus className="w-4 h-4 mr-2" />
                    Add
                  </Button>
                </div>

                {items.length === 0 ? (
                  <div className="text-gray-400">No items yet.</div>
                ) : (
                  <div className="space-y-2">
                    {items.map((it) => (
                      <div key={it.id} className="glass-card p-3 flex items-center justify-between">
                        <div>
                          <div className="text-sm text-gray-200">
                            <span className="text-gray-500 mr-2">{it.item_type}</span>
                            {it.value}
                          </div>
                          <div className="text-xs text-gray-500">
                            weight {it.weight.toFixed(2)} • {it.enabled ? "enabled" : "disabled"}
                          </div>
                        </div>
                        <List className="w-4 h-4 text-gray-600" />
                      </div>
                    ))}
                  </div>
                )}
              </>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}


