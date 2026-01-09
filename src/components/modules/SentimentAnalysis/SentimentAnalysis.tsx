import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import { TrendingUp, TrendingDown, BarChart3 } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface SentimentDataPoint {
  timestamp: number;
  sentiment: number;
}

export default function SentimentAnalysis() {
  const [selectedTicker, setSelectedTicker] = useState("AAPL");
  const [days, setDays] = useState(7);
  const [sentimentData, setSentimentData] = useState<SentimentDataPoint[]>([]);
  const [aggregatedSentiment, setAggregatedSentiment] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(false);
  const [tickers, setTickers] = useState<string[]>([]);
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadTickers();
  }, []);

  useEffect(() => {
    if (selectedTicker) {
      loadSentimentData();
    }
  }, [selectedTicker, days]);

  const loadTickers = async () => {
    try {
      const result = await invoke<any[]>("get_stock_tickers", { index: null });
      setTickers(result.slice(0, 50).map((t) => t.symbol));
    } catch (err) {
      errorHandler.showError("Failed to load tickers", err);
    }
  };

  const loadSentimentData = async () => {
    try {
      setLoading(true);
      const result = await invoke<[number, number][]>("get_news_sentiment", {
        ticker: selectedTicker,
        days,
      });

      const data = result.map(([ts, sentiment]) => ({
        timestamp: ts,
        sentiment,
      }));

      setSentimentData(data);
    } catch (err) {
      errorHandler.showError("Failed to load sentiment data", err);
    } finally {
      setLoading(false);
    }
  };

  const loadAggregatedSentiment = async (tickerList: string[]) => {
    try {
      const result = await invoke<Record<string, number>>("get_aggregated_sentiment", {
        tickers: tickerList,
      });
      setAggregatedSentiment(result);
    } catch (err) {
      errorHandler.showError("Failed to load aggregated sentiment", err);
    }
  };

  const chartData = useMemo(() => {
    return sentimentData.map((d) => ({
      date: new Date(d.timestamp * 1000).toLocaleDateString(),
      sentiment: d.sentiment,
    }));
  }, [sentimentData]);

  const avgSentiment = useMemo(() => {
    if (sentimentData.length === 0) return 0;
    const sum = sentimentData.reduce((acc, d) => acc + d.sentiment, 0);
    return sum / sentimentData.length;
  }, [sentimentData]);

  const getSentimentColor = (sentiment: number) => {
    if (sentiment > 0.2) return "text-neon-cyan";
    if (sentiment < -0.2) return "text-neon-red";
    return "text-neon-amber";
  };

  const getSentimentLabel = (sentiment: number) => {
    if (sentiment > 0.5) return "Very Positive";
    if (sentiment > 0.2) return "Positive";
    if (sentiment < -0.5) return "Very Negative";
    if (sentiment < -0.2) return "Negative";
    return "Neutral";
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-200">Sentiment Analysis</h2>
          <p className="text-sm text-gray-400">Analyze news sentiment for stocks</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card title="Ticker Selection" subtitle="Select ticker to analyze">
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Ticker</label>
              <select
                value={selectedTicker}
                onChange={(e) => setSelectedTicker(e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan font-mono"
              >
                {tickers.map((ticker) => (
                  <option key={ticker} value={ticker}>
                    {ticker}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Days</label>
              <select
                value={days}
                onChange={(e) => setDays(Number(e.target.value))}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
              >
                <option value={7}>7 Days</option>
                <option value={14}>14 Days</option>
                <option value={30}>30 Days</option>
                <option value={90}>90 Days</option>
              </select>
            </div>
          </div>
        </Card>

        <Card title="Average Sentiment" subtitle={selectedTicker}>
          {loading ? (
            <div className="text-center py-8 text-gray-400">Loading...</div>
          ) : (
            <div className="space-y-4">
              <div>
                <div className={`text-4xl font-bold ${getSentimentColor(avgSentiment)}`}>
                  {avgSentiment.toFixed(3)}
                </div>
                <div className="text-sm text-gray-400 mt-1">{getSentimentLabel(avgSentiment)}</div>
              </div>
              <div className="flex items-center gap-2">
                {avgSentiment >= 0 ? (
                  <TrendingUp className="w-5 h-5 text-neon-cyan" />
                ) : (
                  <TrendingDown className="w-5 h-5 text-neon-red" />
                )}
                <span className="text-sm text-gray-400">
                  Based on {sentimentData.length} data points
                </span>
              </div>
            </div>
          )}
        </Card>

        <Card title="Sentiment Trend" subtitle="Over time">
          {loading ? (
            <div className="text-center py-8 text-gray-400">Loading...</div>
          ) : chartData.length > 0 ? (
            <div className="h-48">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={chartData}>
                  <XAxis dataKey="date" tick={{ fill: "#9ca3af", fontSize: 10 }} />
                  <YAxis
                    domain={[-1, 1]}
                    tick={{ fill: "#9ca3af", fontSize: 10 }}
                  />
                  <Tooltip
                    contentStyle={{ background: "rgba(0,0,0,0.8)", border: "1px solid rgba(255,255,255,0.1)" }}
                    labelStyle={{ color: "#e5e7eb" }}
                  />
                  <Line
                    type="monotone"
                    dataKey="sentiment"
                    stroke="#22d3ee"
                    strokeWidth={2}
                    dot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">No data available</div>
          )}
        </Card>
      </div>

      {chartData.length > 0 && (
        <Card title="Sentiment Timeline" subtitle={`${selectedTicker} - Last ${days} days`}>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <XAxis dataKey="date" tick={{ fill: "#9ca3af", fontSize: 10 }} />
                <YAxis
                  domain={[-1, 1]}
                  tick={{ fill: "#9ca3af", fontSize: 10 }}
                />
                <Tooltip
                  contentStyle={{ background: "rgba(0,0,0,0.8)", border: "1px solid rgba(255,255,255,0.1)" }}
                  labelStyle={{ color: "#e5e7eb" }}
                />
                <Line
                  type="monotone"
                  dataKey="sentiment"
                  stroke="#22d3ee"
                  strokeWidth={2}
                  dot={{ fill: "#22d3ee", r: 3 }}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </Card>
      )}
    </div>
  );
}
