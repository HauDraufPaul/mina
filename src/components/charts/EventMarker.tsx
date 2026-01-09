import { TemporalEventMarker } from "./MarketChart";

export interface EventMarkerProps {
  event: TemporalEventMarker;
  onClick?: (event: TemporalEventMarker) => void;
}

export default function EventMarker({ event, onClick }: EventMarkerProps) {
  const getColor = () => {
    if (event.sentimentScore > 0.2) return "text-neon-cyan";
    if (event.sentimentScore < -0.2) return "text-neon-red";
    return "text-neon-amber";
  };

  const getSeverityIcon = () => {
    if (event.severity > 0.7) return "🔴";
    if (event.severity > 0.4) return "🟡";
    return "🟢";
  };

  return (
    <div
      className={`p-2 rounded border border-white/10 hover:border-white/20 cursor-pointer transition-colors ${getColor()}`}
      onClick={() => onClick?.(event)}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs">{getSeverityIcon()}</span>
            <span className="text-xs font-semibold">{event.title}</span>
          </div>
          <div className="text-xs text-gray-400">
            {new Date(event.timestamp * 1000).toLocaleString()}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            {event.eventType} • Sentiment: {event.sentimentScore.toFixed(2)}
          </div>
        </div>
      </div>
    </div>
  );
}
