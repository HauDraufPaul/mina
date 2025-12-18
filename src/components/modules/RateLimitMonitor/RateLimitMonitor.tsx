import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import { Timer, AlertCircle, TrendingUp } from "lucide-react";

interface RateLimitBucket {
  name: string;
  capacity: number;
  tokens: number;
  refill_rate: number;
  refill_interval: number;
  last_refill: number;
}

export default function RateLimitMonitor() {
  const [buckets, setBuckets] = useState<RateLimitBucket[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadBuckets();
    const interval = setInterval(() => {
      loadBuckets();
      // Refill buckets periodically
      buckets.forEach((bucket) => {
        invoke("refill_rate_limit_bucket", { name: bucket.name }).catch(console.error);
      });
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadBuckets = async () => {
    try {
      const data = await invoke<RateLimitBucket[]>("list_rate_limit_buckets");
      setBuckets(data);
      setLoading(false);
    } catch (error) {
      console.error("Failed to load rate limit buckets:", error);
      setLoading(false);
    }
  };

  const getUsagePercentage = (bucket: RateLimitBucket) => {
    return ((bucket.capacity - bucket.tokens) / bucket.capacity) * 100;
  };

  const getStatusColor = (percentage: number) => {
    if (percentage >= 90) return "text-neon-red";
    if (percentage >= 70) return "text-neon-amber";
    return "text-neon-green";
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleTimeString();
  };

  if (loading) {
    return <div className="text-center">Loading rate limit data...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Rate Limit Monitor
        </h1>
        <p className="text-gray-400">API rate limiting analytics and monitoring</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <Card title="Total Buckets" subtitle="Rate limit buckets">
          <div className="text-3xl font-bold text-neon-cyan">{buckets.length}</div>
        </Card>
        <Card title="Average Usage" subtitle="Across all buckets">
          <div className="text-3xl font-bold text-neon-amber">
            {buckets.length > 0
              ? Math.round(
                  buckets.reduce((sum, b) => sum + getUsagePercentage(b), 0) /
                    buckets.length
                )
              : 0}
            %
          </div>
        </Card>
        <Card title="At Risk" subtitle="Buckets > 70% usage">
          <div className="text-3xl font-bold text-neon-red">
            {buckets.filter((b) => getUsagePercentage(b) > 70).length}
          </div>
        </Card>
      </div>

      <Card title="Rate Limit Buckets">
        <div className="space-y-4">
          {buckets.length === 0 ? (
            <div className="text-center text-gray-400 py-8">
              <Timer className="w-12 h-12 mx-auto mb-4 text-gray-500" />
              <p>No rate limit buckets configured</p>
            </div>
          ) : (
            buckets.map((bucket, index) => {
              const usage = getUsagePercentage(bucket);
              return (
                <div key={index} className="glass-card p-4">
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                      <Timer className="w-5 h-5 text-neon-cyan" />
                      <span className="font-semibold">{bucket.name}</span>
                    </div>
                    <span className={`text-sm font-semibold ${getStatusColor(usage)}`}>
                      {usage.toFixed(1)}% used
                    </span>
                  </div>
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm text-gray-400">
                      <span>
                        Tokens: {bucket.tokens} / {bucket.capacity}
                      </span>
                      <span>Refill: {bucket.refill_rate}/sec</span>
                    </div>
                    <div className="w-full bg-gray-800 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full transition-all ${
                          usage >= 90
                            ? "bg-neon-red"
                            : usage >= 70
                            ? "bg-neon-amber"
                            : "bg-neon-green"
                        }`}
                        style={{ width: `${usage}%` }}
                      />
                    </div>
                    <div className="flex justify-between text-xs text-gray-500">
                      <span>Last refill: {formatTime(bucket.last_refill)}</span>
                      <span>Interval: {bucket.refill_interval}s</span>
                    </div>
                    {usage >= 90 && (
                      <div className="flex items-center gap-2 text-xs text-neon-red">
                        <AlertCircle className="w-4 h-4" />
                        <span>Warning: Approaching rate limit</span>
                      </div>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </Card>

      <Card title="Usage Patterns">
        <div className="space-y-4">
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-5 h-5 text-neon-green" />
              <span>Total Capacity</span>
            </div>
            <span className="font-mono text-sm">
              {buckets.reduce((sum, b) => sum + b.capacity, 0)}
            </span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <Timer className="w-5 h-5 text-neon-amber" />
              <span>Available Tokens</span>
            </div>
            <span className="font-mono text-sm">
              {buckets.reduce((sum, b) => sum + b.tokens, 0)}
            </span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <AlertCircle className="w-5 h-5 text-neon-red" />
              <span>Total Usage</span>
            </div>
            <span className="font-mono text-sm">
              {buckets.length > 0
                ? Math.round(
                    buckets.reduce((sum, b) => sum + getUsagePercentage(b), 0) /
                      buckets.length
                  )
                : 0}
              %
            </span>
          </div>
        </div>
      </Card>
    </div>
  );
}
