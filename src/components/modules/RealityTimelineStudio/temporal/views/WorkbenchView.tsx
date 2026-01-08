import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import { Plus, Play, RefreshCw } from "lucide-react";

interface FeatureDefinition {
  id: number;
  name: string;
  expression: string;
  description?: string | null;
  created_at: number;
}

interface FeatureValue {
  id: number;
  feature_id: number;
  ts: number;
  value: number;
}

export default function WorkbenchView() {
  const [features, setFeatures] = useState<FeatureDefinition[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [values, setValues] = useState<FeatureValue[]>([]);
  const [loading, setLoading] = useState(true);
  const [computing, setComputing] = useState(false);

  const [newName, setNewName] = useState("Alerts (7d)");
  const [newExpr, setNewExpr] = useState("alerts_count(7)");
  const [newDesc, setNewDesc] = useState("Daily count of alerts (MVP)");

  const load = async () => {
    setLoading(true);
    try {
      const defs = await invoke<FeatureDefinition[]>("temporal_list_feature_definitions");
      setFeatures(defs);
      if (!selectedId && defs.length > 0) setSelectedId(defs[0].id);
    } finally {
      setLoading(false);
    }
  };

  const loadValues = async (featureId: number) => {
    const vals = await invoke<FeatureValue[]>("temporal_list_feature_values", {
      featureId,
      limit: 365,
    });
    setValues(vals);
  };

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    if (selectedId) loadValues(selectedId);
  }, [selectedId]);

  const create = async () => {
    const name = newName.trim();
    const expression = newExpr.trim();
    if (!name || !expression) return;
    await invoke<number>("temporal_create_feature_definition", {
      name,
      expression,
      description: newDesc?.trim() ? newDesc.trim() : null,
    });
    await load();
  };

  const compute = async () => {
    if (!selectedId) return;
    setComputing(true);
    try {
      await invoke<number>("temporal_compute_feature_mvp", { featureId: selectedId, daysBack: 30 });
      await loadValues(selectedId);
    } finally {
      setComputing(false);
    }
  };

  const chartData = useMemo(
    () =>
      values.map((v) => ({
        ts: new Date(v.ts * 1000).toLocaleDateString(),
        value: v.value,
      })),
    [values]
  );

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-200">Workbench</h3>
          <p className="text-sm text-gray-400">Define and compute custom features (MVP DSL)</p>
        </div>
        <Button variant="secondary" onClick={load} disabled={loading}>
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card title="Features" subtitle={loading ? "Loading…" : `${features.length} definitions`}>
          {features.length === 0 ? (
            <div className="text-gray-400">No features yet. Create one below.</div>
          ) : (
            <div className="space-y-2">
              {features.map((f) => (
                <button
                  key={f.id}
                  onClick={() => setSelectedId(f.id)}
                  className={`w-full text-left glass-card p-3 ${
                    selectedId === f.id ? "border border-neon-cyan/40" : ""
                  }`}
                >
                  <div className="text-sm text-gray-200 font-semibold">{f.name}</div>
                  <div className="text-xs text-gray-500 font-mono">{f.expression}</div>
                </button>
              ))}
            </div>
          )}

          <div className="mt-4 pt-4 border-t border-white/10 space-y-2">
            <div className="text-xs text-gray-500">Create feature</div>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
              placeholder="Feature name"
            />
            <select
              value={newExpr}
              onChange={(e) => setNewExpr(e.target.value)}
              className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
            >
              <option value="alerts_count(7)">alerts_count(7)</option>
              <option value="events_count(7)">events_count(7)</option>
              <option value="avg_sentiment(7)">avg_sentiment(7)</option>
              <option value="alerts_count(30)">alerts_count(30)</option>
              <option value="events_count(30)">events_count(30)</option>
              <option value="avg_sentiment(30)">avg_sentiment(30)</option>
            </select>
            <input
              value={newDesc}
              onChange={(e) => setNewDesc(e.target.value)}
              className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
              placeholder="Description (optional)"
            />
            <Button variant="primary" onClick={create} disabled={!newName.trim() || !newExpr.trim()}>
              <Plus className="w-4 h-4 mr-2" />
              Create
            </Button>
          </div>
        </Card>

        <div className="md:col-span-2">
          <Card
            title="Feature Chart"
            subtitle={
              selectedId
                ? `${values.length} points`
                : "Select a feature"
            }
          >
            {!selectedId ? (
              <div className="text-gray-400">Select a feature to view values.</div>
            ) : (
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <Button variant="primary" onClick={compute} disabled={computing}>
                    <Play className="w-4 h-4 mr-2" />
                    {computing ? "Computing…" : "Compute (30d)"}
                  </Button>
                  <Button variant="secondary" onClick={() => loadValues(selectedId)}>
                    Refresh values
                  </Button>
                </div>

                <div className="h-[360px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={chartData}>
                      <XAxis dataKey="ts" tick={{ fill: "#9ca3af", fontSize: 10 }} />
                      <YAxis tick={{ fill: "#9ca3af", fontSize: 10 }} />
                      <Tooltip
                        contentStyle={{ background: "rgba(0,0,0,0.8)", border: "1px solid rgba(255,255,255,0.1)" }}
                        labelStyle={{ color: "#e5e7eb" }}
                      />
                      <Line type="monotone" dataKey="value" stroke="#22d3ee" strokeWidth={2} dot={false} />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}


