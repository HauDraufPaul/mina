import { useState, useEffect } from "react";
import MarketChart from "@/components/charts/MarketChart";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Search, Settings, BarChart3, LineChart } from "lucide-react";

export default function ChartStudio() {
  const [ticker, setTicker] = useState("AAPL");
  const [timeframe, setTimeframe] = useState<"1m" | "5m" | "15m" | "1h" | "1d">("1d");
  const [showEvents, setShowEvents] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");

  const handleTickerChange = (newTicker: string) => {
    const upperTicker = newTicker.trim().toUpperCase();
    if (upperTicker) {
      setTicker(upperTicker);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-200">Chart Studio</h2>
          <p className="text-sm text-gray-400">Advanced financial charting with temporal event overlay</p>
        </div>
      </div>

      <Card title="Chart Controls" subtitle="Configure your chart">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Ticker Symbol</label>
            <div className="flex items-center gap-2">
              <Search className="w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleTickerChange(searchQuery);
                  }
                }}
                placeholder="Enter ticker (e.g., AAPL)"
                className="flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan font-mono"
              />
              <Button variant="primary" onClick={() => handleTickerChange(searchQuery)}>
                Load
              </Button>
            </div>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-2">Timeframe</label>
            <select
              value={timeframe}
              onChange={(e) => setTimeframe(e.target.value as typeof timeframe)}
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
            >
              <option value="1m">1 Minute</option>
              <option value="5m">5 Minutes</option>
              <option value="15m">15 Minutes</option>
              <option value="1h">1 Hour</option>
              <option value="1d">1 Day</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-2">Options</label>
            <div className="space-y-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={showEvents}
                  onChange={(e) => setShowEvents(e.target.checked)}
                  className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
                />
                <span className="text-sm text-gray-300">Show Events</span>
              </label>
            </div>
          </div>

          <div className="flex items-end">
            <div className="text-sm text-gray-400">
              Current: <span className="font-mono text-white">{ticker}</span>
            </div>
          </div>
        </div>
      </Card>

      <Card title="Price Chart" subtitle={`${ticker} - ${timeframe}`}>
        <MarketChart
          ticker={ticker}
          timeframe={timeframe}
          showEvents={showEvents}
          height={500}
        />
      </Card>
    </div>
  );
}
