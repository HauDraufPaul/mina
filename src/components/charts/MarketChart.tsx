import { useEffect, useRef, useState } from "react";
import { createChart, ColorType, IChartApi, ISeriesApi, CandlestickData, Time } from "lightweight-charts";
import { invoke } from "@tauri-apps/api/core";
import { Calendar, Download } from "lucide-react";
import Button from "../ui/Button";
import { useErrorHandler } from "@/utils/errorHandler";
import {
  calculateSMA,
  calculateEMA,
  calculateRSI,
  calculateMACD,
  calculateBollingerBands,
  formatIndicatorData,
  type IndicatorConfig,
  type PricePoint,
} from "./TechnicalIndicators";

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
  indicators?: IndicatorConfig[];
  comparisonTickers?: string[];
  showDrawingTools?: boolean;
  onExport?: () => void;
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
  onEventClick: _onEventClick,
  indicators = [],
  comparisonTickers = [],
  showDrawingTools: _showDrawingTools,
  onExport,
}: MarketChartProps) {
  const errorHandler = useErrorHandler();
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const indicatorSeriesRefs = useRef<Map<string, ISeriesApi<"Line">>>(new Map());
  const comparisonSeriesRefs = useRef<Map<string, ISeriesApi<"Line">>>(new Map());
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
    const candlestickSeries = (chart as any).addCandlestickSeries({
      upColor: "#22d3ee",
      downColor: "#f87171",
      borderVisible: false,
      wickUpColor: "#22d3ee",
      wickDownColor: "#f87171",
    });
    candlestickSeriesRef.current = candlestickSeries;

    // Create volume series
    const volumeSeries = (chart as any).addHistogramSeries({
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
      indicatorSeriesRefs.current.clear();
      comparisonSeriesRefs.current.clear();
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

          if (candlestickSeriesRef.current) {
            candlestickSeriesRef.current.setData(candlestickData);
          }
          if (volumeSeriesRef.current) {
            volumeSeriesRef.current.setData(volumeData);
          }
          
          // Calculate and add indicators
          if (indicators.length > 0 && chartRef.current) {
            const pricePoints: PricePoint[] = priceDataResult.map((d) => ({
              time: d.time,
              close: d.close,
              high: d.high,
              low: d.low,
              volume: d.volume,
            }));
            
            for (const indicator of indicators) {
              if (!indicator.visible) continue;
              
              let indicatorData: Array<{ time: number; value: number } | { time: number; upper: number; middle: number; lower: number }> = [];
              let seriesKey = `${indicator.type}_${indicator.period || 20}`;
              
              switch (indicator.type) {
                case "sma": {
                  const values = calculateSMA(pricePoints, indicator.period || 20);
                  indicatorData = formatIndicatorData(indicator, pricePoints, values);
                  break;
                }
                case "ema": {
                  const values = calculateEMA(pricePoints, indicator.period || 20);
                  indicatorData = formatIndicatorData(indicator, pricePoints, values);
                  break;
                }
                case "rsi": {
                  const values = calculateRSI(pricePoints, indicator.period || 14);
                  indicatorData = formatIndicatorData(indicator, pricePoints, values);
                  // RSI should be on separate scale (0-100)
                  seriesKey = `rsi_${indicator.period || 14}`;
                  break;
                }
                case "macd": {
                  const macdResult = calculateMACD(pricePoints);
                  indicatorData = formatIndicatorData(indicator, pricePoints, macdResult);
                  break;
                }
                case "bollinger": {
                  const bbResult = calculateBollingerBands(pricePoints, indicator.period || 20);
                  indicatorData = formatIndicatorData(indicator, pricePoints, bbResult);
                  break;
                }
              }
              
              if (indicatorData.length > 0) {
                // Get or create series
                let series = indicatorSeriesRefs.current.get(seriesKey);
                if (!series && chartRef.current) {
                  if (indicator.type === "rsi") {
                    // RSI on separate price scale
                    (chartRef.current as any).addPriceScale("rsi");
                    series = (chartRef.current as any).addLineSeries({
                      color: indicator.color || "#fbbf24",
                      lineWidth: 2,
                      priceScaleId: "rsi",
                      priceFormat: {
                        type: "price",
                        precision: 2,
                        minMove: 0.01,
                      },
                    });
                  } else if (indicator.type === "bollinger") {
                    // Bollinger Bands need area series (simplified as lines for now)
                    series = (chartRef.current as any).addLineSeries({
                      color: indicator.color || "#3b82f6",
                      lineWidth: 1,
                      lineStyle: 2, // Dashed
                    });
                  } else {
                    series = (chartRef.current as any).addLineSeries({
                      color: indicator.color || "#22d3ee",
                      lineWidth: 2,
                    });
                  }
                  if (series) {
                    indicatorSeriesRefs.current.set(seriesKey, series);
                  }
                }
                
                const finalSeries = indicatorSeriesRefs.current.get(seriesKey);
                if (finalSeries && indicatorData.length > 0) {
                  const formattedData = indicatorData.map((d) => {
                    const value = 'value' in d ? d.value : ('middle' in d ? d.middle : 0);
                    return {
                      time: d.time as Time,
                      value,
                    };
                  });
                  finalSeries.setData(formattedData);
                }
              }
            }
          }
          
          // Load comparison tickers
          if (comparisonTickers.length > 0 && chartRef.current) {
            for (const compTicker of comparisonTickers) {
              if (compTicker === ticker) continue;
              
              try {
                const compData = await invoke<OHLCVData[]>("get_chart_data", {
                  ticker: compTicker,
                  fromTs,
                  toTs: now,
                  interval: timeframe,
                });
                
                if (compData && compData.length > 0) {
                  // Normalize to percentage change for comparison
                  const firstPrice = compData[0].close;
                  const normalizedData = compData.map((d) => ({
                    time: d.time as Time,
                    value: ((d.close - firstPrice) / firstPrice) * 100,
                  }));
                  
                  let compSeries = comparisonSeriesRefs.current.get(compTicker);
                  if (!compSeries && chartRef.current) {
                    compSeries = (chartRef.current as any).addLineSeries({
                      color: `#${Math.floor(Math.random() * 16777215).toString(16)}`,
                      lineWidth: 1,
                      lineStyle: 1, // Dotted
                      title: compTicker,
                    });
                    if (compSeries) {
                      comparisonSeriesRefs.current.set(compTicker, compSeries);
                    }
                  }
                  
                  const finalCompSeries = comparisonSeriesRefs.current.get(compTicker);
                  if (finalCompSeries && normalizedData.length > 0) {
                    finalCompSeries.setData(normalizedData);
                  }
                }
              } catch (err) {
                errorHandler.showError(`Failed to load comparison data for ${compTicker}`, err);
              }
            }
          }
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
            if (candlestickSeriesRef.current) {
              (candlestickSeriesRef.current as any).setMarkers(markers);
            }
          }
        }
      } catch (err) {
        errorHandler.showError("Failed to load chart data", err);
        setError(err instanceof Error ? err.message : "Failed to load chart data");
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [ticker, timeframe, showEvents, indicators, comparisonTickers]);
  
  // Handle export
  const handleExport = () => {
    if (!chartRef.current) return;
    
    // Export as PNG
    try {
      const canvas = (chartRef.current as any).takeScreenshot();
      if (canvas) {
        const dataUrl = canvas.toDataURL("image/png");
        const link = document.createElement("a");
        link.download = `${ticker}_${timeframe}_${Date.now()}.png`;
        link.href = dataUrl;
        link.click();
      }
    } catch (err) {
      errorHandler.showError("Failed to export chart", err);
    }
    
    if (onExport) {
      onExport();
    }
  };

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
          {onExport && (
            <Button variant="secondary" onClick={handleExport} className="!px-2 !py-1">
              <Download className="w-4 h-4" />
            </Button>
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
