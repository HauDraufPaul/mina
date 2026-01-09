import { useEffect, useRef, useState } from "react";
import { createChart, ColorType, IChartApi, ISeriesApi, CandlestickData, LineData, Time } from "lightweight-charts";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, TrendingUp, TrendingDown } from "lucide-react";
import Button from "../ui/Button";

export interface TemporalEventMarker {
  id: number;
  title: string;
  timestamp: number;
  eventType: string;
  severity: number;
  sentimentScore: number;
}

export interface MarketChartProps {
  ticker: string;
  timeframe?: "1m" | "5m" | "15m" | "1h" | "1d";
  showEvents?: boolean;
  height?: number;
  onEventClick?: (event: TemporalEventMarker) => void;
}

interface OHLCVData {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export default function MarketChart({
  ticker,
  timeframe = "1d",
  showEvents = true,
  height = 400,
  onEventClick,
}: MarketChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const eventMarkersRef = useRef<{ id: string; time: Time; position: "aboveBar" | "belowBar"; color: string; shape: "circle" | "arrowUp" | "arrowDown"; text: string }[]>([]);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<TemporalEventMarker[]>([]);
  const [priceData, setPriceData] = useState<OHLCVData[]>([]);

  // Initialize chart
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "#0a0a0a" },
        textColor: "#9ca3af",
      },
      grid: {
        vertLines: { color: "rgba(255, 255, 255, 0.05)" },
        horzLines: { color: "rgba(255, 255, 255, 0.05)" },
      },
      width: chartContainerRef.current.clientWidth,
      height: height,
      timeScale: {
        timeVisible: true,
        secondsVisible: timeframe === "1m" || timeframe === "5m",
      },
      rightPriceScale: {
        borderColor: "rgba(255, 255, 255, 0.1)",
      },
    });

    chartRef.current = chart;

    // Create candlestick series
    const candlestickSeries = chart.addCandlestickSeries({
      upColor: "#22d3ee",
      downColor: "#f87171",
      borderVisible: false,
      wickUpColor: "#22d3ee",
      wickDownColor: "#f87171",
    });
    candlestickSeriesRef.current = candlestickSeries;

    // Create volume series
    const volumeSeries = chart.addHistogramSeries({
      color: "#3b82f6",
      priceFormat: {
        type: "volume",
      },
      priceScaleId: "",
      scaleMargins: {
        top: 0.8,
        bottom: 0,
      },
    });
    volumeSeriesRef.current = volumeSeries;

    // Handle resize
    const handleResize = () => {
      if (chartContainerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
    };
  }, [timeframe, height]);

  // Load market data
  useEffect(() => {
    if (!ticker || !chartRef.current || !candlestickSeriesRef.current || !volumeSeriesRef.current) return;

    const loadData = async () => {
      setLoading(true);
      setError(null);

      try {
        const now = Math.floor(Date.now() / 1000);
        const daysBack = timeframe === "1d" ? 30 : timeframe === "1h" ? 7 : timeframe === "15m" ? 1 : 0.5;
        const fromTs = now - daysBack * 24 * 3600;

        // Load price data (will be implemented in backend)
        // For now, use mock data structure
        const priceDataResult = await invoke<OHLCVData[]>("get_chart_data", {
          ticker,
          fromTs,
          toTs: now,
          interval: timeframe,
        }).catch(() => {
          // Fallback: return empty array if command doesn't exist yet
          return [];
        });

        if (priceDataResult && priceDataResult.length > 0) {
          setPriceData(priceDataResult);

          // Format data for TradingView
          const candlestickData: CandlestickData[] = priceDataResult.map((d) => ({
            time: d.time as Time,
            open: d.open,
            high: d.high,
            low: d.low,
            close: d.close,
          }));

          const volumeData = priceDataResult.map((d) => ({
            time: d.time as Time,
            value: d.volume,
            color: d.close >= d.open ? "rgba(34, 211, 238, 0.3)" : "rgba(248, 113, 113, 0.3)",
          }));

          candlestickSeriesRef.current.setData(candlestickData);
          volumeSeriesRef.current.setData(volumeData);
        }

        // Load temporal events if enabled
        if (showEvents) {
          const eventsResult = await invoke<TemporalEventMarker[]>("get_events_for_chart", {
            ticker,
            fromTs,
            toTs: now,
          }).catch(() => {
            return [];
          });

          setEvents(eventsResult);

          // Add event markers to chart
          if (eventsResult.length > 0 && chartRef.current) {
            const markers = eventsResult.map((event) => {
              const sentimentColor = event.sentimentScore > 0.2 ? "#22d3ee" : event.sentimentScore < -0.2 ? "#f87171" : "#fbbf24";
              return {
                time: event.timestamp as Time,
                position: event.sentimentScore > 0 ? ("aboveBar" as const) : ("belowBar" as const),
                color: sentimentColor,
                shape: event.severity > 0.7 ? ("arrowUp" as const) : ("circle" as const),
                text: event.title,
                id: `event-${event.id}`,
              };
            });

            eventMarkersRef.current = markers;
            candlestickSeriesRef.current.setMarkers(markers);
          }
        }
      } catch (err) {
        console.error("Failed to load chart data:", err);
        setError(err instanceof Error ? err.message : "Failed to load chart data");
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [ticker, timeframe, showEvents]);

  if (!ticker) {
    return (
      <div className="flex items-center justify-center h-[400px] text-gray-400">
        <div className="text-center">
          <p className="text-lg mb-2">No ticker selected</p>
          <p className="text-sm">Enter a ticker symbol to view chart</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h3 className="text-lg font-semibold text-gray-200">{ticker}</h3>
          <div className="flex items-center gap-1 text-sm text-gray-400">
            <Calendar className="w-4 h-4" />
            <span>{timeframe}</span>
          </div>
          {events.length > 0 && (
            <div className="flex items-center gap-1 text-sm text-gray-400">
              <span>{events.length} events</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          {priceData.length > 0 && priceData[priceData.length - 1] && (
            <div className="flex items-center gap-2">
              <span className="text-sm text-gray-400">Last:</span>
              <span className={`text-sm font-mono ${priceData[priceData.length - 1].close >= priceData[priceData.length - 1].open ? "text-neon-cyan" : "text-neon-red"}`}>
                ${priceData[priceData.length - 1].close.toFixed(2)}
              </span>
            </div>
          )}
        </div>
      </div>

      {error && (
        <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
          {error}
        </div>
      )}

      {loading && (
        <div className="flex items-center justify-center h-[400px] text-gray-400">
          <div className="text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-neon-cyan mx-auto mb-2"></div>
            <p>Loading chart data...</p>
          </div>
        </div>
      )}

      <div ref={chartContainerRef} className="w-full" style={{ height: `${height}px` }} />

      {events.length > 0 && (
        <div className="mt-2 p-2 bg-white/5 rounded text-xs text-gray-400">
          <div className="flex items-center gap-4">
            <span>Event markers:</span>
            <span className="flex items-center gap-1">
              <div className="w-3 h-3 rounded-full bg-neon-cyan"></div>
              <span>Positive</span>
            </span>
            <span className="flex items-center gap-1">
              <div className="w-3 h-3 rounded-full bg-neon-red"></div>
              <span>Negative</span>
            </span>
            <span className="flex items-center gap-1">
              <div className="w-3 h-3 rounded-full bg-neon-amber"></div>
              <span>Neutral</span>
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
