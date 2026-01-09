import { useState, useEffect } from "react";
import MarketChart from "@/components/charts/MarketChart";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { Search, Settings, BarChart3, LineChart, Plus, X } from "lucide-react";
import type { IndicatorConfig } from "@/components/charts/TechnicalIndicators";

export default function ChartStudio() {
  const [ticker, setTicker] = useState("AAPL");
  const [timeframe, setTimeframe] = useState<"1m" | "5m" | "15m" | "1h" | "1d">("1d");
  const [showEvents, setShowEvents] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [indicators, setIndicators] = useState<IndicatorConfig[]>([]);
  const [comparisonTickers, setComparisonTickers] = useState<string[]>([]);
  const [comparisonInput, setComparisonInput] = useState("");
  const [showIndicatorPanel, setShowIndicatorPanel] = useState(false);

  const handleTickerChange = (newTicker: string) => {
    const upperTicker = newTicker.trim().toUpperCase();
    if (upperTicker) {
      setTicker(upperTicker);
    }
  };
  
  const addIndicator = (type: IndicatorConfig["type"], period: number = 20) => {
    const newIndicator: IndicatorConfig = {
      type,
      period,
      color: type === "sma" ? "#22d3ee" : type === "ema" ? "#fbbf24" : type === "rsi" ? "#a78bfa" : "#3b82f6",
      visible: true,
    };
    setIndicators([...indicators, newIndicator]);
  };
  
  const removeIndicator = (index: number) => {
    setIndicators(indicators.filter((_, i) => i !== index));
  };
  
  const toggleIndicator = (index: number) => {
    setIndicators(indicators.map((ind, i) => 
      i === index ? { ...ind, visible: !ind.visible } : ind
    ));
  };
  
  const addComparison = () => {
    const upperTicker = comparisonInput.trim().toUpperCase();
    if (upperTicker && upperTicker !== ticker && !comparisonTickers.includes(upperTicker)) {
      setComparisonTickers([...comparisonTickers, upperTicker]);
      setComparisonInput("");
    }
  };
  
  const removeComparison = (tickerToRemove: string) => {
    setComparisonTickers(comparisonTickers.filter(t => t !== tickerToRemove));
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
          
          <div>
            <label className="block text-sm text-gray-400 mb-2">Comparison</label>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={comparisonInput}
                onChange={(e) => setComparisonInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    addComparison();
                  }
                }}
                placeholder="Add ticker"
                className="flex-1 px-3 py-1.5 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan font-mono text-sm"
              />
              <Button variant="primary" onClick={addComparison} className="!px-2 !py-1">
                <Plus className="w-4 h-4" />
              </Button>
            </div>
            {comparisonTickers.length > 0 && (
              <div className="flex flex-wrap gap-2 mt-2">
                {comparisonTickers.map((t) => (
                  <span
                    key={t}
                    className="inline-flex items-center gap-1 px-2 py-1 bg-white/5 border border-white/10 rounded text-xs text-gray-300"
                  >
                    {t}
                    <button
                      onClick={() => removeComparison(t)}
                      className="hover:text-red-400"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>

          <div className="flex items-end">
            <div className="text-sm text-gray-400">
              Current: <span className="font-mono text-white">{ticker}</span>
            </div>
          </div>
        </div>
      </Card>

      <Card 
        title="Price Chart" 
        subtitle={`${ticker} - ${timeframe}`}
        actions={
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              onClick={() => setShowIndicatorPanel(!showIndicatorPanel)}
              className="!px-3 !py-1.5"
            >
              <Settings className="w-4 h-4 mr-2" />
              Indicators
            </Button>
          </div>
        }
      >
        {showIndicatorPanel && (
          <div className="mb-4 p-3 bg-white/5 border border-white/10 rounded">
            <div className="flex items-center justify-between mb-3">
              <span className="text-sm font-semibold text-gray-300">Technical Indicators</span>
              <div className="flex gap-2">
                <Button variant="secondary" onClick={() => addIndicator("sma", 20)} className="!px-2 !py-1 text-xs">
                  SMA 20
                </Button>
                <Button variant="secondary" onClick={() => addIndicator("ema", 20)} className="!px-2 !py-1 text-xs">
                  EMA 20
                </Button>
                <Button variant="secondary" onClick={() => addIndicator("rsi", 14)} className="!px-2 !py-1 text-xs">
                  RSI 14
                </Button>
                <Button variant="secondary" onClick={() => addIndicator("macd")} className="!px-2 !py-1 text-xs">
                  MACD
                </Button>
                <Button variant="secondary" onClick={() => addIndicator("bollinger", 20)} className="!px-2 !py-1 text-xs">
                  BB 20
                </Button>
              </div>
            </div>
            {indicators.length > 0 && (
              <div className="space-y-2">
                {indicators.map((indicator, idx) => (
                  <div key={idx} className="flex items-center justify-between p-2 bg-white/5 rounded">
                    <div className="flex items-center gap-2">
                      <input
                        type="checkbox"
                        checked={indicator.visible}
                        onChange={() => toggleIndicator(idx)}
                        className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
                      />
                      <span className="text-sm text-gray-300">
                        {indicator.type.toUpperCase()}
                        {indicator.period && ` (${indicator.period})`}
                      </span>
                      <div
                        className="w-3 h-3 rounded-full"
                        style={{ backgroundColor: indicator.color }}
                      />
                    </div>
                    <button
                      onClick={() => removeIndicator(idx)}
                      className="text-gray-400 hover:text-red-400"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
        
        <MarketChart
          ticker={ticker}
          timeframe={timeframe}
          showEvents={showEvents}
          height={500}
          indicators={indicators}
          comparisonTickers={comparisonTickers}
          onExport={() => console.log("Chart exported")}
        />
      </Card>
    </div>
  );
}
