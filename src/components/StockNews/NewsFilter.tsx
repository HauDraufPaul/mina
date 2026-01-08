import { useStockNewsStore } from "../../stores/stockNewsStore";
import { X } from "lucide-react";

export default function NewsFilter() {
  const {
    selectedIndex,
    selectedSource,
    selectedTickers,
    timeRange,
    setSelectedIndex,
    setSelectedSource,
    toggleTicker,
    setTimeRange,
    clearFilters,
  } = useStockNewsStore();

  // Get unique sources from tickers
  const sources = Array.from(
    new Set(["Bloomberg", "Reuters", "MarketWatch", "Yahoo Finance", "CNBC", "WSJ"])
  );

  const hasFilters =
    selectedIndex || selectedSource || selectedTickers.length > 0 || timeRange !== "24h";

  return (
    <div className="p-4 border-b border-white/10 space-y-3">
      {/* Index Filter */}
      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-400 font-mono w-20">INDEX:</label>
        <div className="flex gap-2">
          <button
            onClick={() => setSelectedIndex(null)}
            className={`px-3 py-1 rounded text-xs font-mono transition-colors
                     ${
                       !selectedIndex
                         ? "bg-neon-cyan text-black"
                         : "bg-white/5 text-gray-400 hover:bg-white/10"
                     }`}
          >
            ALL
          </button>
          <button
            onClick={() => setSelectedIndex("SP500")}
            className={`px-3 py-1 rounded text-xs font-mono transition-colors
                     ${
                       selectedIndex === "SP500"
                         ? "bg-neon-cyan text-black"
                         : "bg-white/5 text-gray-400 hover:bg-white/10"
                     }`}
          >
            S&P 500
          </button>
          <button
            onClick={() => setSelectedIndex("DAX")}
            className={`px-3 py-1 rounded text-xs font-mono transition-colors
                     ${
                       selectedIndex === "DAX"
                         ? "bg-neon-cyan text-black"
                         : "bg-white/5 text-gray-400 hover:bg-white/10"
                     }`}
          >
            DAX
          </button>
        </div>
      </div>

      {/* Time Range Filter */}
      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-400 font-mono w-20">TIME:</label>
        <div className="flex gap-2 flex-wrap">
          {(["1h", "6h", "24h", "7d", "30d", "all"] as const).map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={`px-3 py-1 rounded text-xs font-mono transition-colors
                       ${
                         timeRange === range
                           ? "bg-neon-cyan text-black"
                           : "bg-white/5 text-gray-400 hover:bg-white/10"
                       }`}
            >
              {range.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {/* Source Filter */}
      <div className="flex items-center gap-2">
        <label className="text-xs text-gray-400 font-mono w-20">SOURCE:</label>
        <div className="flex gap-2 flex-wrap">
          <button
            onClick={() => setSelectedSource(null)}
            className={`px-3 py-1 rounded text-xs font-mono transition-colors
                     ${
                       !selectedSource
                         ? "bg-neon-cyan text-black"
                         : "bg-white/5 text-gray-400 hover:bg-white/10"
                     }`}
          >
            ALL
          </button>
          {sources.map((source) => (
            <button
              key={source}
              onClick={() => setSelectedSource(source)}
              className={`px-3 py-1 rounded text-xs font-mono transition-colors
                       ${
                         selectedSource === source
                           ? "bg-neon-cyan text-black"
                           : "bg-white/5 text-gray-400 hover:bg-white/10"
                       }`}
            >
              {source.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {/* Selected Tickers */}
      {selectedTickers.length > 0 && (
        <div className="flex items-center gap-2">
          <label className="text-xs text-gray-400 font-mono w-20">TICKERS:</label>
          <div className="flex gap-2 flex-wrap">
            {selectedTickers.map((ticker) => (
              <button
                key={ticker}
                onClick={() => toggleTicker(ticker)}
                className="px-3 py-1 rounded text-xs font-mono bg-neon-cyan text-black
                         hover:bg-neon-cyan/80 transition-colors flex items-center gap-1"
              >
                {ticker}
                <X className="w-3 h-3" />
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Clear Filters */}
      {hasFilters && (
        <div className="flex items-center gap-2">
          <button
            onClick={clearFilters}
            className="text-xs text-gray-400 hover:text-white transition-colors
                     underline font-mono"
          >
            Clear all filters
          </button>
        </div>
      )}
    </div>
  );
}

