import { useState, useEffect, useMemo } from "react";
import { useMarketData, MarketPrice } from "@/hooks/useMarketData";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Search, TrendingUp, TrendingDown, Plus, X } from "lucide-react";
import { useStockNewsStore } from "@/stores/stockNewsStore";
import { useErrorHandler } from "@/utils/errorHandler";

export default function MarketDataHub() {
  const [searchQuery, setSearchQuery] = useState("");
  const [watchlistTickers, setWatchlistTickers] = useState<string[]>([]);
  const [_allPrices, _setAllPrices] = useState<MarketPrice[]>([]);
  const [_loading, _setLoading] = useState(false);
  const { tickers } = useStockNewsStore();
  const errorHandler = useErrorHandler();

  const { prices, loading: pricesLoading, fetchPrices, formatPrice, formatChange } = useMarketData({
    tickers: watchlistTickers,
    autoSubscribe: true,
    refreshInterval: 60000,
  });

  useEffect(() => {
    loadWatchlistTickers();
  }, []);

  const loadWatchlistTickers = async () => {
    try {
      // Get tickers from watchlist or use default
      if (tickers.length > 0) {
        setWatchlistTickers(tickers.slice(0, 20).map((t) => t.symbol));
      } else {
        // Default watchlist
        setWatchlistTickers(["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA"]);
      }
    } catch (err) {
      errorHandler.showError("Failed to load watchlist", err);
    }
  };

  const handleAddTicker = async () => {
    const ticker = searchQuery.trim().toUpperCase();
    if (!ticker || watchlistTickers.includes(ticker)) return;

    setWatchlistTickers([...watchlistTickers, ticker]);
    setSearchQuery("");

    // Fetch price for new ticker
    try {
      await fetchPrices([ticker]);
    } catch (err) {
      errorHandler.showError("Failed to fetch price", err);
    }
  };

  const handleRemoveTicker = (ticker: string) => {
    setWatchlistTickers(watchlistTickers.filter((t) => t !== ticker));
  };

  const sortedPrices = useMemo(() => {
    return watchlistTickers
      .map((ticker) => prices.get(ticker))
      .filter((p): p is MarketPrice => p !== undefined)
      .sort((a, b) => b.change_percent - a.change_percent);
  }, [watchlistTickers, prices]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-200">Market Data Hub</h2>
          <p className="text-sm text-gray-400">Real-time market prices and watchlist management</p>
        </div>
      </div>

      <Card title="Watchlist" subtitle={`${watchlistTickers.length} tickers`}>
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleAddTicker();
                  }
                }}
                placeholder="Enter ticker symbol (e.g., AAPL)"
                className="w-full pl-10 pr-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
              />
            </div>
            <Button variant="primary" onClick={handleAddTicker} disabled={!searchQuery.trim()}>
              <Plus className="w-4 h-4 mr-2" />
              Add
            </Button>
          </div>

          <div className="flex flex-wrap gap-2">
            {watchlistTickers.map((ticker) => (
              <div
                key={ticker}
                className="flex items-center gap-2 px-3 py-1 bg-white/5 border border-white/10 rounded"
              >
                <span className="text-sm font-mono">{ticker}</span>
                <button
                  onClick={() => handleRemoveTicker(ticker)}
                  className="text-gray-400 hover:text-white transition-colors"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>
        </div>
      </Card>

      <Card title="Market Prices" subtitle={pricesLoading ? "Loading..." : `${sortedPrices.length} prices`}>
        {sortedPrices.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            <p>No prices available</p>
            <p className="text-sm mt-2">Add tickers to your watchlist to see prices</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-white/10">
                  <th className="text-left py-2 px-4 text-sm text-gray-400">Ticker</th>
                  <th className="text-right py-2 px-4 text-sm text-gray-400">Price</th>
                  <th className="text-right py-2 px-4 text-sm text-gray-400">Change</th>
                  <th className="text-right py-2 px-4 text-sm text-gray-400">Volume</th>
                </tr>
              </thead>
              <tbody>
                {sortedPrices.map((price) => (
                  <tr key={price.ticker} className="border-b border-white/5 hover:bg-white/5">
                    <td className="py-2 px-4 font-mono text-sm">{price.ticker}</td>
                    <td className="py-2 px-4 text-right font-mono text-sm">{formatPrice(price)}</td>
                    <td
                      className={`py-2 px-4 text-right font-mono text-sm ${
                        price.change >= 0 ? "text-neon-cyan" : "text-neon-red"
                      }`}
                    >
                      <div className="flex items-center justify-end gap-1">
                        {price.change >= 0 ? (
                          <TrendingUp className="w-3 h-3" />
                        ) : (
                          <TrendingDown className="w-3 h-3" />
                        )}
                        <span>{formatChange(price)}</span>
                      </div>
                    </td>
                    <td className="py-2 px-4 text-right font-mono text-xs text-gray-400">
                      {price.volume.toLocaleString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
}
