import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WidgetProps } from "./WidgetRegistry";
import { Calendar, AlertTriangle } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface EconomicEvent {
  id: number;
  name: string;
  country: string;
  event_type: string;
  scheduled_at: number;
  actual_value: number | null;
  forecast_value: number | null;
  previous_value: number | null;
  impact_score: number;
  created_at: number;
  updated_at: number;
}

export default function CalendarWidget({ config }: WidgetProps) {
  const daysAhead = (config.daysAhead as number) || 7;
  const country = config.country as string | null;
  const showImpact = config.showImpact !== false;
  const [events, setEvents] = useState<EconomicEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadEvents();
  }, [daysAhead, country]);

  const loadEvents = async () => {
    try {
      setLoading(true);
      const now = Math.floor(Date.now() / 1000);
      const fromTs = now;
      const toTs = now + daysAhead * 24 * 3600;

      const result = await invoke<EconomicEvent[]>("list_economic_events", {
        fromTs,
        toTs,
        country,
        eventType: null,
      });

      // Filter to only upcoming events
      const upcoming = result
        .filter((e) => e.actual_value === null)
        .sort((a, b) => a.scheduled_at - b.scheduled_at)
        .slice(0, 5);

      setEvents(upcoming);
    } catch (err) {
      errorHandler.showError("Failed to load events", err);
    } finally {
      setLoading(false);
    }
  };

  const getImpactColor = (score: number) => {
    if (score >= 0.7) return "text-neon-red";
    if (score >= 0.4) return "text-neon-amber";
    return "text-neon-cyan";
  };

  const getImpactLabel = (score: number) => {
    if (score >= 0.7) return "High";
    if (score >= 0.4) return "Med";
    return "Low";
  };

  if (loading) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">Loading events...</div>
    );
  }

  if (events.length === 0) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">
        <Calendar className="w-6 h-6 mx-auto mb-2 text-gray-500" />
        <p>No upcoming events</p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {events.map((event) => (
        <div
          key={event.id}
          className="p-2 bg-white/5 rounded border border-white/10 hover:border-white/20 transition-colors"
        >
          <div className="flex items-start justify-between mb-1">
            <div className="flex-1 min-w-0">
              <div className="text-xs font-semibold truncate">{event.name}</div>
              <div className="text-xs text-gray-400">
                {event.country} • {new Date(event.scheduled_at * 1000).toLocaleDateString()}
              </div>
            </div>
            {showImpact && (
              <div className={`text-xs font-semibold ${getImpactColor(event.impact_score)}`}>
                {getImpactLabel(event.impact_score)}
              </div>
            )}
          </div>
          {event.forecast_value !== null && (
            <div className="text-xs text-gray-500 mt-1">
              Forecast: {event.forecast_value}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

