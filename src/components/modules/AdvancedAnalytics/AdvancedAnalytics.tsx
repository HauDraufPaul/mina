import { useState } from "react";
import Card from "../../ui/Card";
import { BarChart3, TrendingUp, PieChart, Activity } from "lucide-react";

export default function AdvancedAnalytics() {
  const [selectedMetric, setSelectedMetric] = useState("cpu");

  const metrics = [
    { id: "cpu", label: "CPU Usage", color: "neon-cyan" },
    { id: "memory", label: "Memory Usage", color: "neon-green" },
    { id: "disk", label: "Disk Usage", color: "neon-amber" },
    { id: "network", label: "Network Traffic", color: "neon-red" },
  ];

  // Mock data for visualization
  const generateData = (count: number) => {
    return Array.from({ length: count }, (_, i) => ({
      time: new Date(Date.now() - (count - i) * 60000).toLocaleTimeString(),
      value: Math.random() * 100,
    }));
  };

  const chartData = generateData(20);

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
          {metrics.map((metric) => (
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

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Time Series Analysis" subtitle="Trend over time">
          <div className="space-y-4">
            <div className="flex items-center gap-2 mb-4">
              <TrendingUp className="w-5 h-5 text-neon-cyan" />
              <span className="font-semibold">
                {metrics.find((m) => m.id === selectedMetric)?.label}
              </span>
            </div>
            <div className="space-y-2">
              {chartData.map((point, index) => (
                <div key={index} className="flex items-center gap-3">
                  <div className="w-20 text-xs text-gray-400 font-mono">
                    {point.time}
                  </div>
                  <div className="flex-1 bg-gray-800 rounded-full h-3 relative">
                    <div
                      className="bg-neon-cyan h-3 rounded-full transition-all"
                      style={{ width: `${point.value}%` }}
                    />
                  </div>
                  <div className="w-16 text-xs text-right font-mono">
                    {point.value.toFixed(1)}%
                  </div>
                </div>
              ))}
            </div>
          </div>
        </Card>

        <Card title="Statistical Summary" subtitle="Key metrics">
          <div className="space-y-4">
            <div className="flex items-center justify-between p-3 glass-card">
              <span className="text-gray-400">Mean</span>
              <span className="font-mono font-semibold text-neon-cyan">
                {(
                  chartData.reduce((sum, d) => sum + d.value, 0) /
                  chartData.length
                ).toFixed(2)}%
              </span>
            </div>
            <div className="flex items-center justify-between p-3 glass-card">
              <span className="text-gray-400">Max</span>
              <span className="font-mono font-semibold text-neon-red">
                {Math.max(...chartData.map((d) => d.value)).toFixed(2)}%
              </span>
            </div>
            <div className="flex items-center justify-between p-3 glass-card">
              <span className="text-gray-400">Min</span>
              <span className="font-mono font-semibold text-neon-green">
                {Math.min(...chartData.map((d) => d.value)).toFixed(2)}%
              </span>
            </div>
            <div className="flex items-center justify-between p-3 glass-card">
              <span className="text-gray-400">Std Dev</span>
              <span className="font-mono font-semibold text-neon-amber">
                {(
                  Math.sqrt(
                    chartData.reduce(
                      (sum, d) =>
                        sum +
                        Math.pow(
                          d.value -
                            chartData.reduce((s, x) => s + x.value, 0) /
                              chartData.length,
                          2
                        ),
                      0
                    ) / chartData.length
                  )
                ).toFixed(2)}%
              </span>
            </div>
          </div>
        </Card>
      </div>

      <Card title="Distribution Analysis">
        <div className="space-y-4">
          <div className="flex items-center gap-2 mb-4">
            <PieChart className="w-5 h-5 text-neon-green" />
            <span className="font-semibold">Value Distribution</span>
          </div>
          <div className="grid grid-cols-4 gap-4">
            {[
              { label: "0-25%", count: chartData.filter((d) => d.value < 25).length },
              {
                label: "25-50%",
                count: chartData.filter((d) => d.value >= 25 && d.value < 50).length,
              },
              {
                label: "50-75%",
                count: chartData.filter((d) => d.value >= 50 && d.value < 75).length,
              },
              {
                label: "75-100%",
                count: chartData.filter((d) => d.value >= 75).length,
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
    </div>
  );
}
