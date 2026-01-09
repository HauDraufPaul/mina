import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WidgetProps } from "./WidgetRegistry";

interface TickerPrice {
  ticker: string;
  price: number;
  change: number;
  changePercent: number;
}

export default function WatchlistWidget({ config }: WidgetProps) {
  const [prices, setPrices] = useState<TickerPrice[]>([]);
  const [loading, setLoading] = useState(true);
  const tickers = (config.tickers as string[]) || ["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA"];

  useEffect(() => {
    const loadPrices = async () => {
      setLoading(true);
      try {
        const priceData = await invoke<any[]>("get_market_prices", { tickers }).catch(() => []);
        
        const items: TickerPrice[] = priceData.map((p) => ({
          ticker: p.ticker || "",
          price: p.price || 0,
          change: p.change || 0,
          changePercent: p.changePercent || 0,
        }));

        setPrices(items);
      } catch (err) {
        console.error("Failed to load prices:", err);
      } finally {
        setLoading(false);
      }
    };

    loadPrices();
    
    // Refresh every 30 seconds
    const interval = setInterval(loadPrices, 30000);
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
    <div className="space-y-1 h-full overflow-y-auto">
      {prices.map((item) => (
        <div
          key={item.ticker}
          className="flex items-center justify-between p-2 bg-white/5 border border-white/10 rounded hover:bg-white/10 transition-colors"
        >
          <span className="font-mono text-sm font-semibold text-gray-200">{item.ticker}</span>
          <div className="flex items-center gap-3">
            <span className="text-sm font-mono text-gray-300">${item.price.toFixed(2)}</span>
            <span
              className={`text-xs font-mono ${
                item.changePercent >= 0 ? "text-green-400" : "text-red-400"
              }`}
            >
              {item.changePercent >= 0 ? "+" : ""}
              {item.changePercent.toFixed(2)}%
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

