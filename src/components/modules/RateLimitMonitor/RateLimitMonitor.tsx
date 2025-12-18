import { useState, useEffect } from "react";
import Card from "../../ui/Card";
import { Timer, AlertCircle, TrendingUp } from "lucide-react";

interface RateLimitBucket {
  name: string;
  capacity: number;
  tokens: number;
  refillRate: number;
  lastRefill: number;
}

export default function RateLimitMonitor() {
  const [buckets, setBuckets] = useState<RateLimitBucket[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate rate limit buckets
    const mockBuckets: RateLimitBucket[] = [
      {
        name: "API Requests",
        capacity: 100,
        tokens: 75,
        refillRate: 10,
        lastRefill: Date.now(),
      },
      {
        name: "WebSocket Messages",
        capacity: 1000,
        tokens: 850,
        refillRate: 100,
        lastRefill: Date.now(),
      },
      {
        name: "Database Queries",
        capacity: 500,
        tokens: 320,
        refillRate: 50,
        lastRefill: Date.now(),
      },
    ];

    setBuckets(mockBuckets);
    setLoading(false);
  }, []);

  const getUsagePercentage = (bucket: RateLimitBucket) => {
    return ((bucket.capacity - bucket.tokens) / bucket.capacity) * 100;
  };

  const getStatusColor = (percentage: number) => {
    if (percentage >= 90) return "text-neon-red";
    if (percentage >= 70) return "text-neon-amber";
    return "text-neon-green";
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
            {Math.round(
              buckets.reduce((sum, b) => sum + getUsagePercentage(b), 0) /
                buckets.length
            )}
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
          {buckets.map((bucket, index) => {
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
                    <span>Tokens: {bucket.tokens} / {bucket.capacity}</span>
                    <span>Refill: {bucket.refillRate}/sec</span>
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
                  {usage >= 90 && (
                    <div className="flex items-center gap-2 text-xs text-neon-red">
                      <AlertCircle className="w-4 h-4" />
                      <span>Warning: Approaching rate limit</span>
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </Card>

      <Card title="Usage Patterns">
        <div className="space-y-4">
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-5 h-5 text-neon-green" />
              <span>Peak Usage Time</span>
            </div>
            <span className="font-mono text-sm">14:00 - 16:00</span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <Timer className="w-5 h-5 text-neon-amber" />
              <span>Average Requests/Min</span>
            </div>
            <span className="font-mono text-sm">45 req/min</span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <div className="flex items-center gap-2">
              <AlertCircle className="w-5 h-5 text-neon-red" />
              <span>Rate Limit Hits (24h)</span>
            </div>
            <span className="font-mono text-sm">3 hits</span>
          </div>
        </div>
      </Card>
    </div>
  );
}
