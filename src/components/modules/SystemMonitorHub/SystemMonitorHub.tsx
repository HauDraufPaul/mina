import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import { Activity, Cpu, HardDrive, Wifi, MemoryStick } from "lucide-react";
import ProcessList from "./ProcessList";

interface SystemMetrics {
  cpu: {
    usage: number;
    cores: number;
    frequency: number;
  };
  memory: {
    total: number;
    used: number;
    free: number;
    usage: number;
  };
  disk: {
    total: number;
    used: number;
    free: number;
    usage: number;
  };
  network: {
    rx: number;
    tx: number;
    rxSpeed: number;
    txSpeed: number;
  };
}

export default function SystemMonitorHub() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        const data = await invoke<SystemMetrics>("get_system_metrics");
        setMetrics(data);
        setLoading(false);
      } catch (error) {
        console.error("Failed to fetch metrics:", error);
        setLoading(false);
      }
    };

    fetchMetrics();
    const interval = setInterval(fetchMetrics, 1000);

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return <div className="text-center">Loading system metrics...</div>;
  }

  if (!metrics) {
    return <div className="text-center text-red-500">Failed to load metrics</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          System Monitor Hub
        </h1>
        <p className="text-gray-400">Real-time system performance monitoring</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card title="CPU Usage" subtitle={`${metrics.cpu.cores} cores`}>
          <div className="flex items-center gap-4">
            <Cpu className="w-12 h-12 text-neon-cyan" />
            <div className="flex-1">
              <div className="text-3xl font-bold text-neon-cyan">
                {metrics.cpu.usage.toFixed(1)}%
              </div>
              <div className="w-full bg-gray-800 rounded-full h-2 mt-2">
                <div
                  className="bg-neon-cyan h-2 rounded-full transition-all"
                  style={{ width: `${metrics.cpu.usage}%` }}
                />
              </div>
            </div>
          </div>
        </Card>

        <Card title="Memory Usage">
          <div className="flex items-center gap-4">
            <MemoryStick className="w-12 h-12 text-neon-green" />
            <div className="flex-1">
              <div className="text-3xl font-bold text-neon-green">
                {metrics.memory.usage.toFixed(1)}%
              </div>
              <div className="text-sm text-gray-400 mt-1">
                {(metrics.memory.used / 1024 / 1024 / 1024).toFixed(2)} GB /{" "}
                {(metrics.memory.total / 1024 / 1024 / 1024).toFixed(2)} GB
              </div>
              <div className="w-full bg-gray-800 rounded-full h-2 mt-2">
                <div
                  className="bg-neon-green h-2 rounded-full transition-all"
                  style={{ width: `${metrics.memory.usage}%` }}
                />
              </div>
            </div>
          </div>
        </Card>

        <Card title="Disk Usage">
          <div className="flex items-center gap-4">
            <HardDrive className="w-12 h-12 text-neon-amber" />
            <div className="flex-1">
              <div className="text-3xl font-bold text-neon-amber">
                {metrics.disk.usage.toFixed(1)}%
              </div>
              <div className="text-sm text-gray-400 mt-1">
                {(metrics.disk.used / 1024 / 1024 / 1024).toFixed(2)} GB /{" "}
                {(metrics.disk.total / 1024 / 1024 / 1024).toFixed(2)} GB
              </div>
              <div className="w-full bg-gray-800 rounded-full h-2 mt-2">
                <div
                  className="bg-neon-amber h-2 rounded-full transition-all"
                  style={{ width: `${metrics.disk.usage}%` }}
                />
              </div>
            </div>
          </div>
        </Card>

        <Card title="Network">
          <div className="flex items-center gap-4">
            <Wifi className="w-12 h-12 text-neon-green" />
            <div className="flex-1">
              <div className="text-lg font-semibold text-neon-green">
                ↓ {(metrics.network.rxSpeed / 1024 / 1024).toFixed(2)} MB/s
              </div>
              <div className="text-lg font-semibold text-neon-cyan mt-1">
                ↑ {(metrics.network.txSpeed / 1024 / 1024).toFixed(2)} MB/s
              </div>
            </div>
          </div>
        </Card>
      </div>

      <ProcessList />
    </div>
  );
}

