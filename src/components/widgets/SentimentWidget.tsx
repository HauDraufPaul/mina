import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WidgetProps } from "./WidgetRegistry";
import { useErrorHandler } from "@/utils/errorHandler";

interface SentimentData {
  ticker: string;
  score: number;
  count: number;
}

export default function SentimentWidget({ config }: WidgetProps) {
  const errorHandler = useErrorHandler();
  const [sentiment, setSentiment] = useState<SentimentData[]>([]);
  const [loading, setLoading] = useState(true);
  const tickers = (config.tickers as string[]) || ["AAPL", "MSFT", "GOOGL"];
  const showCount = config.showCount !== false;

  useEffect(() => {
    const loadSentiment = async () => {
      setLoading(true);
      try {
        const sentimentData = await invoke<Record<string, number>>("get_aggregated_sentiment", {
          tickers,
        }).catch(() => ({}));

        const items: SentimentData[] = tickers.map((ticker) => ({
          ticker,
          score: sentimentData[ticker] || 0,
          count: 0, // Count would need to come from backend
        }));

        setSentiment(items);
      } catch (err) {
        errorHandler.showError("Failed to load sentiment", err);
      } finally {
        setLoading(false);
      }
    };

    loadSentiment();
    
    // Refresh every 5 minutes
    const interval = setInterval(loadSentiment, 5 * 60 * 1000);
    return () => clearInterval(interval);
  }, [tickers.join(",")]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-neon-cyan"></div>
      </div>
    );
  }

  return (
    <div className="space-y-3 h-full overflow-y-auto">
      {sentiment.map((item) => (
        <div key={item.ticker} className="p-3 bg-white/5 border border-white/10 rounded-lg">
          <div className="flex items-center justify-between mb-2">
            <span className="font-mono text-sm font-semibold text-gray-200">{item.ticker}</span>
            {showCount && item.count > 0 && (
              <span className="text-xs text-gray-400">{item.count} items</span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <div className="flex-1 h-2 bg-gray-800 rounded-full overflow-hidden">
              <div
                className={`h-full transition-all ${
                  item.score > 0.2
                    ? "bg-green-500"
                    : item.score < -0.2
                    ? "bg-red-500"
                    : "bg-gray-500"
                }`}
                style={{
                  width: `${Math.abs(item.score) * 100}%`,
                  marginLeft: item.score < 0 ? `${(1 - Math.abs(item.score)) * 100}%` : "0%",
                }}
              />
            </div>
            <span
              className={`text-sm font-mono w-12 text-right ${
                item.score > 0.2
                  ? "text-green-400"
                  : item.score < -0.2
                  ? "text-red-400"
                  : "text-gray-400"
              }`}
            >
              {item.score > 0 ? "+" : ""}
              {(item.score * 100).toFixed(0)}%
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

