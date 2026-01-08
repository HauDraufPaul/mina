import { useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import Tabs from "@/components/ui/Tabs";
import TimelineView from "./views/TimelineView";
import GraphView from "./views/GraphView";
import AlertsView from "./views/AlertsView";
import WatchlistsView from "./views/WatchlistsView";
import SearchView from "./views/SearchView";
import BacktestsView from "./views/BacktestsView";
import WorkbenchView from "./views/WorkbenchView";

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
    const params = new URLSearchParams(location.search);
    params.set("tab", next);
    navigate({ pathname: location.pathname, search: params.toString() }, { replace: true });
  };

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h2 className="text-2xl font-bold phosphor-glow-cyan">Temporal Engine</h2>
        <p className="text-gray-400">
          OSINT timeline, entity graph, alerts, and research workflows (local-first)
        </p>
      </div>

      <Tabs items={tabItems} activeTab={activeTab} onTabChange={onTabChange} />

      {activeTab === "timeline" && <TimelineView />}
      {activeTab === "graph" && <GraphView />}
      {activeTab === "alerts" && <AlertsView />}
      {activeTab === "watchlists" && <WatchlistsView />}
      {activeTab === "search" && <SearchView />}
      {activeTab === "backtests" && <BacktestsView />}
      {activeTab === "workbench" && <WorkbenchView />}
    </div>
  );
}


