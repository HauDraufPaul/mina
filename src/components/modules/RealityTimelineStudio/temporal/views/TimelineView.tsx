import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import { RefreshCw, Layers, ExternalLink, BarChart3 } from "lucide-react";
import ImpactAnalysis from "../../../PortfolioManager/ImpactAnalysis";
import { DataSet } from "vis-data";
import { Timeline } from "vis-timeline/standalone";
import "vis-timeline/styles/vis-timeline-graph2d.min.css";

interface TemporalEvent {
  id: number;
  title: string;
  summary: string;
  start_ts: number;
  end_ts: number;
  event_type: string;
  confidence: number;
  severity: number;
  novelty_score: number;
  volume_score: number;
  sentiment_score: number;
  cluster_key: string;
}

interface TemporalEvidence {
  event_id: number;
  rss_item_id: number;
  weight: number;
  snippet?: string | null;
}

export default function TimelineView() {
  const timelineContainerId = "mina-temporal-timeline";
  const [events, setEvents] = useState<TemporalEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedEvent, setSelectedEvent] = useState<TemporalEvent | null>(null);
  const [evidence, setEvidence] = useState<TemporalEvidence[]>([]);
  const [loadingEvidence, setLoadingEvidence] = useState(false);
  const [rebuildBusy, setRebuildBusy] = useState(false);
  const [timeline, setTimeline] = useState<any>(null);
  const [selectedPortfolioId, setSelectedPortfolioId] = useState<number | null>(null);
  const [showImpactAnalysis, setShowImpactAnalysis] = useState(false);

  const [portfolios, setPortfolios] = useState<Array<{ id: number; name: string }>>([]);

  const loadEvents = async () => {
    setLoading(true);
    try {
      const data = await invoke<TemporalEvent[]>("temporal_list_events", { limit: 200 });
      setEvents(data);
    } finally {
      setLoading(false);
    }
  };

  const loadPortfolios = async () => {
    try {
      const data = await invoke<Array<{ id: number; name: string; created_at: number }>>("list_portfolios");
      setPortfolios(data);
    } catch (err) {
      console.error("Failed to load portfolios:", err);
    }
  };

  useEffect(() => {
    loadEvents();
    loadPortfolios();
  }, []);

  // Build/update vis-timeline
  useEffect(() => {
    if (loading) return;
    const container = document.getElementById(timelineContainerId);
    if (!container) return;

    const groups = new DataSet(
      Array.from(
        new Set(events.map((e) => (e.title.split(":")[0] || "misc").trim()))
      )
        .slice(0, 50)
        .map((label) => ({ id: label, content: label }))
    );

    const items = new DataSet(
      events.slice(0, 300).map((e) => {
        const entity = (e.title.split(":")[0] || "misc").trim();
        return {
          id: e.id,
          group: entity,
          content: `<div style="font-weight:600">${escapeHtml(e.title)}</div>`,
          start: new Date(e.start_ts * 1000),
          end: e.end_ts && e.end_ts !== e.start_ts ? new Date(e.end_ts * 1000) : undefined,
        };
      })
    );

    if (!timeline) {
      const tl = new Timeline(container, items, groups, {
        stack: true,
        showCurrentTime: true,
        zoomMin: 1000 * 60 * 60 * 6,
        zoomMax: 1000 * 60 * 60 * 24 * 30,
        maxHeight: 520,
      } as any);
      tl.on("select", (props: any) => {
        const id = props?.items?.[0];
        if (id) {
          const evt = events.find((x) => x.id === id);
          if (evt) openEvent(evt);
        }
      });
      setTimeline(tl);
    } else {
      timeline.setGroups(groups);
      timeline.setItems(items);
    }
  }, [events, loading]);

  const rebuild = async () => {
    setRebuildBusy(true);
    try {
      await invoke<number>("temporal_rebuild_events_mvp", { daysBack: 30 });
      await invoke<number>("temporal_rebuild_search_index", {});
      await loadEvents();
    } finally {
      setRebuildBusy(false);
    }
  };

  const openEvent = async (evt: TemporalEvent) => {
    setSelectedEvent(evt);
    setLoadingEvidence(true);
    try {
      const ev = await invoke<TemporalEvidence[]>("temporal_list_event_evidence", { eventId: evt.id });
      setEvidence(ev);
    } finally {
      setLoadingEvidence(false);
    }
  };

  const formatTs = (ts: number) => new Date(ts * 1000).toLocaleString();

  const topEvents = useMemo(() => events.slice(0, 60), [events]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-200">Event Timeline</h3>
          <p className="text-sm text-gray-400">
            Clusters RSS items into daily entity-centric events (MVP clustering)
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" onClick={loadEvents} disabled={loading}>
            <RefreshCw className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button variant="primary" onClick={rebuild} disabled={rebuildBusy}>
            <Layers className="w-4 h-4 mr-2" />
            {rebuildBusy ? "Rebuilding…" : "Rebuild Events"}
          </Button>
        </div>
      </div>

      <Card title="Timeline" subtitle={loading ? "Loading…" : "Click an item to open details"}>
        <div
          id={timelineContainerId}
          className="w-full rounded-lg border border-white/10 bg-black/30"
          style={{ height: 520 }}
        />
      </Card>

      <Card title="Recent Events" subtitle={loading ? "Loading…" : `${events.length} events (list view)`}>
        {loading ? (
          <div className="text-gray-400">Loading events…</div>
        ) : topEvents.length === 0 ? (
          <div className="text-gray-400">No temporal events yet. Click “Rebuild Events”.</div>
        ) : (
          <div className="space-y-3">
            {topEvents.map((evt) => (
              <button
                key={evt.id}
                onClick={() => openEvent(evt)}
                className="w-full text-left glass-card p-4 hover:bg-white/5 transition-colors"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="min-w-0">
                    <div className="font-semibold text-neon-cyan truncate">{evt.title}</div>
                    <div className="text-xs text-gray-500 mt-1">
                      {formatTs(evt.start_ts)} → {formatTs(evt.end_ts)} • vol {evt.volume_score.toFixed(0)} • novelty{" "}
                      {evt.novelty_score.toFixed(2)} • sent {evt.sentiment_score.toFixed(2)}
                    </div>
                    <div className="text-sm text-gray-300 mt-2 line-clamp-2">{evt.summary}</div>
                  </div>
                  <span className="text-xs text-gray-500 whitespace-nowrap">{evt.event_type}</span>
                </div>
              </button>
            ))}
          </div>
        )}
      </Card>

      <Modal
        isOpen={!!selectedEvent}
        onClose={() => {
          setSelectedEvent(null);
          setEvidence([]);
        }}
        title={selectedEvent ? `Event #${selectedEvent.id}` : "Event"}
      >
        {selectedEvent && (
          <div className="space-y-4">
            <div className="text-sm text-gray-300">{selectedEvent.summary}</div>
            <div className="text-xs text-gray-500">
              Cluster: <span className="font-mono text-neon-cyan">{selectedEvent.cluster_key}</span>
            </div>

            <Card title="Evidence" subtitle={loadingEvidence ? "Loading…" : `${evidence.length} linked articles`}>
              {loadingEvidence ? (
                <div className="text-gray-400">Loading evidence…</div>
              ) : evidence.length === 0 ? (
                <div className="text-gray-400">No evidence linked.</div>
              ) : (
                <div className="space-y-3">
                  {evidence.slice(0, 50).map((ev) => (
                    <div key={`${ev.event_id}-${ev.rss_item_id}`} className="glass-card p-3">
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <div className="text-xs text-gray-500">rss_item_id: {ev.rss_item_id}</div>
                          {ev.snippet && <div className="text-sm text-gray-300 mt-1">{ev.snippet}</div>}
                        </div>
                        <a
                          className="text-neon-cyan hover:underline text-xs flex items-center gap-1"
                          onClick={async (e) => {
                            e.preventDefault();
                            // We only have rss_item_id here; open in reader happens in the main module later.
                          }}
                          href="#"
                        >
                          <ExternalLink className="w-3 h-3" />
                          Open
                        </a>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </Card>

            <Card title="Portfolio Impact Analysis" subtitle="Analyze how this event affects your portfolios">
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Select Portfolio</label>
                  <select
                    value={selectedPortfolioId || ""}
                    onChange={(e) => {
                      const portfolioId = e.target.value ? parseInt(e.target.value) : null;
                      setSelectedPortfolioId(portfolioId);
                      setShowImpactAnalysis(portfolioId !== null);
                    }}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
                  >
                    <option value="">Select a portfolio...</option>
                    {portfolios.map((portfolio) => (
                      <option key={portfolio.id} value={portfolio.id}>
                        {portfolio.name}
                      </option>
                    ))}
                  </select>
                </div>

                {showImpactAnalysis && selectedPortfolioId && selectedEvent && (
                  <div className="mt-4">
                    <ImpactAnalysis
                      portfolioId={selectedPortfolioId}
                      eventId={selectedEvent.id}
                      priceChanges={{}}
                    />
                  </div>
                )}
              </div>
            </Card>
          </div>
        )}
      </Modal>
    </div>
  );
}

function escapeHtml(input: string) {
  return input
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}


