import { useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Tabs from "@/components/ui/Tabs";
import Card from "@/components/ui/Card";
import TimelineView from "./views/TimelineView";
import GraphView from "./views/GraphView";
import AlertsView from "./views/AlertsView";
import WatchlistsView from "./views/WatchlistsView";
import SearchView from "./views/SearchView";
import BacktestsView from "./views/BacktestsView";
import WorkbenchView from "./views/WorkbenchView";
import { AlertCircle } from "lucide-react";

type TemporalTab =
  | "timeline"
  | "graph"
  | "alerts"
  | "watchlists"
  | "search"
  | "backtests"
  | "workbench";

function getTabFromQuery(search: string): TemporalTab | null {
  const params = new URLSearchParams(search);
  const tab = params.get("tab");
  if (
    tab === "timeline" ||
    tab === "graph" ||
    tab === "alerts" ||
    tab === "watchlists" ||
    tab === "search" ||
    tab === "backtests" ||
    tab === "workbench"
  ) {
    return tab;
  }
  return null;
}

export default function TemporalEngine() {
  const location = useLocation();
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = useState<TemporalTab>("timeline");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const tabFromQuery = getTabFromQuery(location.search);
    if (tabFromQuery) {
      setActiveTab(tabFromQuery);
    }
  }, [location.search]);

  const tabItems = useMemo(
    () => [
      { id: "timeline", label: "Timeline" },
      { id: "graph", label: "Graph" },
      { id: "alerts", label: "Alerts" },
      { id: "watchlists", label: "Watchlists" },
      { id: "search", label: "Search" },
      { id: "backtests", label: "Backtests" },
      { id: "workbench", label: "Workbench" },
    ],
    []
  );

  const onTabChange = (tabId: string) => {
    const next = tabId as TemporalTab;
    setActiveTab(next);
    setError(null); // Clear errors when switching tabs
    const params = new URLSearchParams(location.search);
    params.set("tab", next);
    navigate({ pathname: location.pathname, search: params.toString() }, { replace: true });
  };

  // Error boundary component wrapper
  const renderTabContent = () => {
    try {
      switch (activeTab) {
        case "timeline":
          return <TimelineView />;
        case "graph":
          return <GraphView />;
        case "alerts":
          return <AlertsView />;
        case "watchlists":
          return <WatchlistsView />;
        case "search":
          return <SearchView />;
        case "backtests":
          return <BacktestsView />;
        case "workbench":
          return <WorkbenchView />;
        default:
          return <TimelineView />;
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "An unknown error occurred";
      setError(errorMessage);
      return (
        <Card title="Error" subtitle="Failed to load Temporal Engine view">
          <div className="flex items-center gap-3 text-red-400">
            <AlertCircle className="w-5 h-5" />
            <div>
              <p className="font-semibold">Error loading {activeTab} view</p>
              <p className="text-sm text-gray-400 mt-1">{errorMessage}</p>
            </div>
          </div>
        </Card>
      );
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold phosphor-glow-cyan">Temporal Engine</h2>
        <p className="text-gray-400">
          OSINT timeline, entity graph, alerts, and research workflows (local-first)
        </p>
      </div>

      {error && (
        <Card title="Error" subtitle="An error occurred">
          <div className="flex items-center gap-3 text-red-400">
            <AlertCircle className="w-5 h-5" />
            <p>{error}</p>
          </div>
        </Card>
      )}

      <Tabs items={tabItems} activeTab={activeTab} onTabChange={onTabChange} />

      {renderTabContent()}
    </div>
  );
}


