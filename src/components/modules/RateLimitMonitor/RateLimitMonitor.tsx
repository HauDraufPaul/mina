import Card from "../../ui/Card";

export default function RateLimitMonitor() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Rate Limit Monitor
        </h1>
        <p className="text-gray-400">API rate limiting analytics</p>
      </div>
      <Card title="Rate Limits">
        <p className="text-gray-400">Rate limit monitoring coming soon...</p>
      </Card>
    </div>
  );
}

