import { useMarketData } from "@/hooks/useMarketData";
import { WidgetProps } from "./WidgetRegistry";
import { TrendingUp, TrendingDown } from "lucide-react";

export default function MarketDataWidget({ config }: WidgetProps) {
  const tickers = (config.tickers as string[]) || ["AAPL", "MSFT", "GOOGL"];
  const showChange = config.showChange !== false;
  const showVolume = config.showVolume === true;

  const { prices, loading, formatPrice } = useMarketData({
    tickers,
    autoSubscribe: true,
    refreshInterval: 60000,
  });

  const priceList = tickers
    .map((ticker) => prices.get(ticker))
    .filter((p) => p !== undefined)
    .slice(0, 5); // Limit to 5 tickers for widget

  if (loading && priceList.length === 0) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">Loading market data...</div>
    );
  }

  if (priceList.length === 0) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">No market data available</div>
    );
  }

  return (
    <div className="space-y-2">
      {priceList.map((price) => (
        <div
          key={price.ticker}
          className="flex items-center justify-between p-2 bg-white/5 rounded border border-white/10 hover:border-white/20 transition-colors"
        >
          <div className="flex-1">
            <div className="font-mono font-semibold text-sm">{price.ticker}</div>
            <div className="text-xs text-gray-400">{formatPrice(price)}</div>
          </div>
          {showChange && (
            <div
              className={`text-xs font-semibold flex items-center gap-1 ${
                price.change >= 0 ? "text-neon-cyan" : "text-neon-red"
              }`}
            >
              {price.change >= 0 ? (
                <TrendingUp className="w-3 h-3" />
              ) : (
                <TrendingDown className="w-3 h-3" />
              )}
              <span>{price.change_percent >= 0 ? "+" : ""}{price.change_percent.toFixed(2)}%</span>
            </div>
          )}
          {showVolume && (
            <div className="text-xs text-gray-400 ml-2">
              {price.volume.toLocaleString()}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

