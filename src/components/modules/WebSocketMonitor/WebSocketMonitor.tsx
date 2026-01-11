import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import { Radio, Activity, Hash } from "lucide-react";

export default function WebSocketMonitor() {
  const [connectionCount, setConnectionCount] = useState(0);
  const [topics, setTopics] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [count, topicList] = await Promise.all([
          invoke<number>("get_ws_connection_count").catch(() => 0),
          invoke<string[]>("get_ws_topics").catch(() => []),
        ]);
        setConnectionCount(count || 0);
        setTopics(topicList || []);
        setLoading(false);
      } catch (error) {
        console.error("Failed to fetch WebSocket data:", error);
        setConnectionCount(0);
        setTopics([]);
        setLoading(false);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 2000);

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return <div className="text-center">Loading WebSocket data...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          WebSocket Monitor
        </h1>
        <p className="text-gray-400">WebSocket connection monitoring and analytics</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card title="Active Connections" subtitle="Current WebSocket connections">
          <div className="flex items-center gap-4">
            <Radio className="w-12 h-12 text-neon-cyan" />
            <div>
              <div className="text-3xl font-bold text-neon-cyan">
                {connectionCount}
              </div>
              <div className="text-sm text-gray-400">connections</div>
            </div>
          </div>
        </Card>

        <Card title="Active Topics" subtitle="Subscribed topics">
          <div className="flex items-center gap-4">
            <Hash className="w-12 h-12 text-neon-green" />
            <div>
              <div className="text-3xl font-bold text-neon-green">
                {topics.length}
              </div>
              <div className="text-sm text-gray-400">topics</div>
            </div>
          </div>
        </Card>
      </div>

      <Card title="Subscribed Topics">
        {topics.length === 0 ? (
          <div className="text-center text-gray-400 py-8">
            <Activity className="w-12 h-12 mx-auto mb-4 text-gray-500" />
            <p>No active topics</p>
          </div>
        ) : (
          <div className="space-y-2">
            {topics.map((topic, index) => (
              <div
                key={index}
                className="glass-card p-3 flex items-center justify-between"
              >
                <span className="font-mono text-sm text-neon-cyan">{topic}</span>
                <span className="text-xs text-gray-400">Active</span>
              </div>
            ))}
          </div>
        )}
      </Card>

      <Card title="WebSocket Server Status">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-gray-400">Server Status</span>
            <span className="text-neon-green flex items-center gap-2">
              <div className="w-2 h-2 bg-neon-green rounded-full animate-pulse" />
              Running
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-400">Broadcast Interval</span>
            <span className="font-mono text-sm">1 second</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-400">Message Buffer</span>
            <span className="font-mono text-sm">1000 messages</span>
          </div>
        </div>
      </Card>
    </div>
  );
}
