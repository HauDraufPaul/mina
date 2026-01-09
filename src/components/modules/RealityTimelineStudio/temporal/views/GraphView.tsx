import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import CytoscapeComponent from "react-cytoscapejs";
import { RefreshCw } from "lucide-react";

interface EntityGraphNode {
  id: string;
  label: string;
  count: number;
}

interface EntityGraphEdge {
  source: string;
  target: string;
  weight: number;
}

interface EntityGraph {
  nodes: EntityGraphNode[];
  edges: EntityGraphEdge[];
}

export default function GraphView() {
  const [graph, setGraph] = useState<EntityGraph | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedEntity, setSelectedEntity] = useState<string | null>(null);
  const [entityEvents, setEntityEvents] = useState<any[]>([]);
  const [loadingEntityEvents, setLoadingEntityEvents] = useState(false);

  const load = async () => {
    setLoading(true);
    try {
      const data = await invoke<EntityGraph>("temporal_get_entity_graph_mvp", {
        daysBack: 30,
        maxNodes: 80,
        maxEdges: 300,
      });
      setGraph(data);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const elements = useMemo(() => {
    if (!graph) return [];
    const els: Array<{ data: Record<string, unknown> }> = [];
    for (const n of graph.nodes) {
      els.push({ data: { id: n.id, label: n.label, count: n.count } });
    }
    for (const e of graph.edges) {
      els.push({ data: { source: e.source, target: e.target, weight: e.weight } });
    }
    return els;
  }, [graph]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-200">Entity Graph</h3>
          <p className="text-sm text-gray-400">
            Co-mention network built from extracted entities across recent articles. Click a node to drill down.\n          </p>
        </div>
        <Button variant="secondary" onClick={load} disabled={loading}>
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      <Card title="Network" subtitle={loading ? "Loading…" : `${graph?.nodes.length ?? 0} nodes • ${graph?.edges.length ?? 0} edges`}>
        {loading ? (
          <div className="text-gray-400">Loading…</div>
        ) : !graph || graph.nodes.length === 0 ? (
          <div className="text-gray-400">
            No graph data yet. Fetch articles and/or run entity extraction.\n          </div>
        ) : (
          <div className="h-[520px]">
            <CytoscapeComponent
              elements={elements}
              style={{ width: "100%", height: "100%" }}
              layout={{ name: "cose", animate: false }}
              stylesheet={[
                {
                  selector: "node",
                  style: {
                    label: "data(label)",
                    color: "#e5e7eb",
                    "text-outline-color": "#000000",
                    "text-outline-width": 2,
                    "font-size": 10,
                    width: "mapData(count, 1, 50, 14, 46)",
                    height: "mapData(count, 1, 50, 14, 46)",
                    "background-color": "#22d3ee",
                  },
                },
                {
                  selector: "edge",
                  style: {
                    width: "mapData(weight, 1, 20, 1, 6)",
                    "line-color": "#64748b",
                    "target-arrow-color": "#64748b",
                    "curve-style": "bezier",
                    opacity: 0.7,
                  },
                },
                {
                  selector: ":selected",
                  style: {
                    "background-color": "#f59e0b",
                    "line-color": "#f59e0b",
                    "target-arrow-color": "#f59e0b",
                  },
                },
              ]}
              cy={(cy: { on: (event: string, selector: string, handler: (evt: { target: { id: () => string } }) => Promise<void>) => void }) => {
                cy.on("tap", "node", async (evt: { target: { id: () => string } }) => {
                  const id = evt.target.id();
                  setSelectedEntity(id);
                  setLoadingEntityEvents(true);
                  try {
                    const evts = await invoke<Array<Record<string, unknown>>>("temporal_list_events", { limit: 500 });
                    setEntityEvents(
                      evts.filter((e) => typeof e.title === "string" && e.title.toLowerCase().startsWith(id.toLowerCase()))
                    );
                  } finally {
                    setLoadingEntityEvents(false);
                  }
                });
              }}
            />
          </div>
        )}
      </Card>

      <Modal isOpen={!!selectedEntity} onClose={() => setSelectedEntity(null)} title={selectedEntity ?? "Entity"}>
        {loadingEntityEvents ? (
          <div className="text-gray-400">Loading events…</div>
        ) : (
          <div className="space-y-2">
            <div className="text-sm text-gray-400">
              Recent events that match this entity label (MVP drill-down).\n            </div>
            {entityEvents.length === 0 ? (
              <div className="text-gray-400">No matching events found yet.</div>
            ) : (
              entityEvents.slice(0, 30).map((e) => (
                <div key={e.id} className="glass-card p-3">
                  <div className="text-neon-cyan font-semibold">{e.title}</div>
                  <div className="text-xs text-gray-500">
                    {new Date(e.start_ts * 1000).toLocaleString()} • vol {Number(e.volume_score ?? 0).toFixed(0)}
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </Modal>
    </div>
  );
}


