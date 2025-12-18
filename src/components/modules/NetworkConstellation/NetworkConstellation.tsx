import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import { Network, Wifi, Activity } from "lucide-react";

interface NetworkInterface {
  name: string;
  received: number;
  transmitted: number;
}

export default function NetworkConstellation() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchInterfaces = async () => {
      try {
        const data = await invoke<NetworkInterface[]>("get_network_interfaces");
        setInterfaces(data);
        setLoading(false);
      } catch (error) {
        console.error("Failed to fetch network interfaces:", error);
        setLoading(false);
      }
    };

    fetchInterfaces();
    const interval = setInterval(fetchInterfaces, 2000);

    return () => clearInterval(interval);
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  if (loading) {
    return <div className="text-center">Loading network interfaces...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Network Constellation
        </h1>
        <p className="text-gray-400">Network monitoring and analysis</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {interfaces.map((iface) => (
          <Card key={iface.name} title={iface.name}>
            <div className="space-y-4">
              <div className="flex items-center gap-4">
                <Wifi className="w-8 h-8 text-neon-cyan" />
                <div className="flex-1">
                  <div className="text-sm text-gray-400 mb-1">Received</div>
                  <div className="text-lg font-semibold text-neon-green">
                    {formatBytes(iface.received)}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <Activity className="w-8 h-8 text-neon-amber" />
                <div className="flex-1">
                  <div className="text-sm text-gray-400 mb-1">Transmitted</div>
                  <div className="text-lg font-semibold text-neon-cyan">
                    {formatBytes(iface.transmitted)}
                  </div>
                </div>
              </div>
            </div>
          </Card>
        ))}
      </div>

      {interfaces.length === 0 && (
        <Card title="No Network Interfaces">
          <p className="text-gray-400 text-center py-8">
            No network interfaces detected
          </p>
        </Card>
      )}
    </div>
  );
}
