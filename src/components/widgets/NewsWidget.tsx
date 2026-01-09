import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WidgetProps } from "./WidgetRegistry";
import { useErrorHandler } from "@/utils/errorHandler";

interface NewsItem {
  id: number;
  title: string;
  summary: string;
  source: string;
  published_at: number;
  sentiment_score: number;
}

export default function NewsWidget({ config }: WidgetProps) {
  const errorHandler = useErrorHandler();
  const [news, setNews] = useState<NewsItem[]>([]);
  const [loading, setLoading] = useState(true);
  const ticker = (config.ticker as string) || null;
  const limit = (config.limit as number) || 5;
  const showSentiment = config.showSentiment !== false;

  useEffect(() => {
    const loadNews = async () => {
      setLoading(true);
      try {
        const now = Math.floor(Date.now() / 1000);
        const fromTs = now - 7 * 24 * 3600; // Last 7 days

        let items: NewsItem[] = [];
        if (ticker) {
          // Get news for specific ticker
          const events = await invoke<any[]>("temporal_list_events", {
            limit: limit * 2,
            fromTs,
            toTs: now,
          });
          
          items = events
            .filter((e) => 
              e.title.toUpperCase().includes(ticker.toUpperCase()) ||
              e.summary.toUpperCase().includes(ticker.toUpperCase())
            )
            .slice(0, limit)
            .map((e) => ({
              id: e.id,
              title: e.title,
              summary: e.summary,
              source: "Temporal Engine",
              published_at: e.start_ts,
              sentiment_score: e.sentiment_score || 0,
            }));
        } else {
          // Get general news
          const events = await invoke<any[]>("temporal_list_events", {
            limit,
            fromTs,
            toTs: now,
          });
          
          items = events.map((e) => ({
            id: e.id,
            title: e.title,
            summary: e.summary,
            source: "Temporal Engine",
            published_at: e.start_ts,
            sentiment_score: e.sentiment_score || 0,
          }));
        }
        
        setNews(items);
      } catch (err) {
        errorHandler.showError("Failed to load news", err);
      } finally {
        setLoading(false);
      }
    };

    loadNews();
  }, [ticker, limit]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-neon-cyan"></div>
      </div>
    );
  }

  return (
    <div className="space-y-2 h-full overflow-y-auto">
      {news.length === 0 ? (
        <div className="text-center text-gray-400 py-4">
          <p className="text-sm">No news available</p>
        </div>
      ) : (
        news.map((item) => (
          <div
            key={item.id}
            className="p-3 bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 transition-colors"
          >
            <div className="flex items-start justify-between gap-2 mb-1">
              <h4 className="text-sm font-semibold text-gray-200 line-clamp-2 flex-1">
                {item.title}
              </h4>
              {showSentiment && (
                <div
                  className={`text-xs px-2 py-0.5 rounded ${
                    item.sentiment_score > 0.2
                      ? "bg-green-500/20 text-green-400"
                      : item.sentiment_score < -0.2
                      ? "bg-red-500/20 text-red-400"
                      : "bg-gray-500/20 text-gray-400"
                  }`}
                >
                  {item.sentiment_score > 0 ? "+" : ""}
                  {(item.sentiment_score * 100).toFixed(0)}%
                </div>
              )}
            </div>
            <p className="text-xs text-gray-400 line-clamp-2 mb-2">{item.summary}</p>
            <div className="flex items-center justify-between text-xs text-gray-500">
              <span>{item.source}</span>
              <span>{new Date(item.published_at * 1000).toLocaleDateString()}</span>
            </div>
          </div>
        ))
      )}
    </div>
  );
}

