import Card from "../../ui/Card";

export default function WebSocketMonitor() {
  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          WebSocket Monitor
        </h1>
        <p className="text-gray-400">WebSocket connection monitoring</p>
      </div>
      <Card title="WebSocket Connections">
        <p className="text-gray-400">WebSocket monitoring coming soon...</p>
      </Card>
    </div>
  );
}

