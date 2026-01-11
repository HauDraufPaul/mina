import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import { BarChart3, TrendingUp, PieChart, Activity } from "lucide-react";

interface AnalyticsMetrics {
  timestamp: number;
  metric_type: string;
  value: number;
  metadata?: string;
}

interface Statistics {
  mean: number;
  min: number;
  max: number;
  std_dev: number;
  count: number;
}

export default function AdvancedAnalytics() {
  const [selectedMetric, setSelectedMetric] = useState("cpu");
  const [metrics, setMetrics] = useState<AnalyticsMetrics[]>([]);
  const [statistics, setStatistics] = useState<Statistics | null>(null);
  const [loading, setLoading] = useState(true);

  const metricTypes = [
    { id: "cpu", label: "CPU Usage", color: "neon-cyan" },
    { id: "memory", label: "Memory Usage", color: "neon-green" },
    { id: "disk", label: "Disk Usage", color: "neon-amber" },
    { id: "network", label: "Network Traffic", color: "neon-red" },
  ];

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, [selectedMetric]);

  const loadData = async () => {
    try {
      setLoading(true);
      const endTime = Math.floor(Date.now() / 1000);
      const startTime = endTime - 3600; // Last hour

      const [metricsData, statsData] = await Promise.all([
        invoke<AnalyticsMetrics[]>("get_metrics", {
          metric_type: selectedMetric,
          start_time: startTime,
          end_time: endTime,
          limit: 20,
        }).catch(() => []),
        invoke<Statistics>("get_statistics", {
          metric_type: selectedMetric,
          start_time: startTime,
          end_time: endTime,
        }).catch(() => null),
      ]);

      setMetrics(metricsData || []);
      setStatistics(statsData);
    } catch (error) {
      console.error("Failed to load analytics:", error);
      setMetrics([]);
      setStatistics(null);
    } finally {
      setLoading(false);
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleTimeString();
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Advanced Analytics
        </h1>
        <p className="text-gray-400">Data visualization and statistical analysis</p>
      </div>

      <Card title="Metric Selection">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          {metricTypes.map((metric) => (
            <button
              key={metric.id}
              onClick={() => setSelectedMetric(metric.id)}
              className={`glass-card p-4 text-center transition-all ${
                selectedMetric === metric.id
                  ? "border-2 border-neon-cyan"
                  : "hover:border border-white/10"
              }`}
            >
              <div className={`text-sm font-semibold text-${metric.color}`}>
                {metric.label}
              </div>
            </button>
          ))}
        </div>
      </Card>

      {loading ? (
        <Card title="Loading...">
          <div className="text-center py-8 text-gray-400">Loading analytics data...</div>
        </Card>
      ) : (
        <>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card title="Time Series Analysis" subtitle="Trend over time">
              <div className="space-y-4">
                <div className="flex items-center gap-2 mb-4">
                  <TrendingUp className="w-5 h-5 text-neon-cyan" />
                  <span className="font-semibold">
                    {metricTypes.find((m) => m.id === selectedMetric)?.label}
                  </span>
                </div>
                <div className="space-y-2">
                  {metrics.length === 0 ? (
                    <div className="text-center text-gray-400 py-8">
                      <Activity className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                      <p className="mb-2">No data available yet</p>
                      <p className="text-sm">Metrics are collected every second. Data will appear here shortly.</p>
                    </div>
                  ) : (
                    metrics.map((point, index) => (
                      <div key={index} className="flex items-center gap-3">
                        <div className="w-20 text-xs text-gray-400 font-mono">
                          {formatTime(point.timestamp)}
                        </div>
                        <div className="flex-1 bg-gray-800 rounded-full h-3 relative">
                          <div
                            className="bg-neon-cyan h-3 rounded-full transition-all"
                            style={{ width: `${Math.min(point.value, 100)}%` }}
                          />
                        </div>
                        <div className="w-16 text-xs text-right font-mono">
                          {point.value.toFixed(1)}%
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </Card>

            <Card title="Statistical Summary" subtitle="Key metrics">
              {statistics ? (
                <div className="space-y-4">
                  <div className="flex items-center justify-between p-3 glass-card">
                    <span className="text-gray-400">Mean</span>
                    <span className="font-mono font-semibold text-neon-cyan">
                      {statistics.mean.toFixed(2)}%
                    </span>
                  </div>
                  <div className="flex items-center justify-between p-3 glass-card">
                    <span className="text-gray-400">Max</span>
                    <span className="font-mono font-semibold text-neon-red">
                      {statistics.max.toFixed(2)}%
                    </span>
                  </div>
                  <div className="flex items-center justify-between p-3 glass-card">
                    <span className="text-gray-400">Min</span>
                    <span className="font-mono font-semibold text-neon-green">
                      {statistics.min.toFixed(2)}%
                    </span>
                  </div>
                  <div className="flex items-center justify-between p-3 glass-card">
                    <span className="text-gray-400">Std Dev</span>
                    <span className="font-mono font-semibold text-neon-amber">
                      {statistics.std_dev.toFixed(2)}%
                    </span>
                  </div>
                  <div className="flex items-center justify-between p-3 glass-card">
                    <span className="text-gray-400">Count</span>
                    <span className="font-mono font-semibold">
                      {statistics.count}
                    </span>
                  </div>
                </div>
              ) : (
                <div className="text-center text-gray-400 py-8">
                  <Activity className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                  <p>No statistics available</p>
                  <p className="text-sm mt-2">Waiting for metric data to be collected...</p>
                </div>
              )}
            </Card>
          </div>

          <Card title="Distribution Analysis">
            <div className="space-y-4">
              <div className="flex items-center gap-2 mb-4">
                <PieChart className="w-5 h-5 text-neon-green" />
                <span className="font-semibold">Value Distribution</span>
              </div>
              {metrics.length > 0 ? (
                <div className="grid grid-cols-4 gap-4">
                  {[
                    { label: "0-25%", count: metrics.filter((d) => d.value < 25).length },
                    {
                      label: "25-50%",
                      count: metrics.filter((d) => d.value >= 25 && d.value < 50).length,
                    },
                    {
                      label: "50-75%",
                      count: metrics.filter((d) => d.value >= 50 && d.value < 75).length,
                    },
                    {
                      label: "75-100%",
                      count: metrics.filter((d) => d.value >= 75).length,
                    },
                  ].map((range, index) => (
                    <div key={index} className="glass-card p-4 text-center">
                      <div className="text-2xl font-bold text-neon-cyan mb-1">
                        {range.count}
                      </div>
                      <div className="text-xs text-gray-400">{range.label}</div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-center text-gray-400 py-8">
                  <Activity className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                  <p>No data available</p>
                  <p className="text-sm mt-2">Metrics are being collected. Check back in a moment.</p>
                </div>
              )}
            </div>
          </Card>

          <Card title="Report Generation">
            <div className="space-y-3">
              <div className="flex items-center gap-2 text-gray-400">
                <Activity className="w-4 h-4" />
                <span>Generate comprehensive analytics report</span>
              </div>
              <button className="glass-button glass-button-primary w-full">
                <BarChart3 className="w-4 h-4 mr-2" />
                Generate Report
              </button>
            </div>
          </Card>
        </>
      )}
    </div>
  );
}
