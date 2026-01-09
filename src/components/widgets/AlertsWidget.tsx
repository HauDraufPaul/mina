import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WidgetProps } from "./WidgetRegistry";
import { Bell, Check, Clock } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";
import { realtimeService } from "@/services/realtimeService";

interface Alert {
  id: number;
  rule_id: number;
  fired_at: number;
  event_id: number | null;
  payload_json: Record<string, unknown>;
  status: string;
  snoozed_until: number | null;
}

export default function AlertsWidget({ config }: WidgetProps) {
  const limit = (config.limit as number) || 10;
  const showUnreadOnly = config.showUnreadOnly === true;
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [loading, setLoading] = useState(true);
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadAlerts();
  }, [limit, showUnreadOnly]);

  useEffect(() => {
    const unsubscribe = realtimeService.subscribe("temporal-alert", () => {
      loadAlerts();
    });
    return () => unsubscribe();
  }, []);

  const loadAlerts = async () => {
    try {
      setLoading(true);
      const allAlerts = await invoke<Alert[]>("temporal_list_alerts", { limit: limit * 2 });
      
      const filtered = showUnreadOnly
        ? allAlerts.filter((a) => a.status === "new")
        : allAlerts;

      setAlerts(filtered.slice(0, limit));
    } catch (err) {
      errorHandler.showError("Failed to load alerts", err);
    } finally {
      setLoading(false);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "new":
        return <Bell className="w-3 h-3 text-neon-cyan" />;
      case "ack":
        return <Check className="w-3 h-3 text-neon-green" />;
      case "snoozed":
        return <Clock className="w-3 h-3 text-neon-amber" />;
      default:
        return null;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "new":
        return "border-neon-cyan/50";
      case "ack":
        return "border-neon-green/50";
      case "snoozed":
        return "border-neon-amber/50";
      default:
        return "border-white/10";
    }
  };

  if (loading) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">Loading alerts...</div>
    );
  }

  if (alerts.length === 0) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">
        <Bell className="w-6 h-6 mx-auto mb-2 text-gray-500" />
        <p>No alerts</p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {alerts.map((alert) => {
        const payload = alert.payload_json;
        const title = (payload.title as string) || (payload.summary as string) || "Alert";
        
        return (
          <div
            key={alert.id}
            className={`p-2 bg-white/5 rounded border ${getStatusColor(alert.status)} hover:border-white/30 transition-colors`}
          >
            <div className="flex items-start gap-2">
              {getStatusIcon(alert.status)}
              <div className="flex-1 min-w-0">
                <div className="text-xs font-semibold truncate">{title}</div>
                <div className="text-xs text-gray-400 mt-0.5">
                  {new Date(alert.fired_at * 1000).toLocaleString()}
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

