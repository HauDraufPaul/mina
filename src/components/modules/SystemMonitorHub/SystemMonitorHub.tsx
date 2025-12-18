import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Cpu, HardDrive, Wifi, MemoryStick, RefreshCw, AlertTriangle, TrendingUp, TrendingDown, Pause, Play } from "lucide-react";
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

// History tracking for sparklines (last 60 data points)
const MAX_HISTORY = 60;

export default function SystemMonitorHub() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isPaused, setIsPaused] = useState(false);
  
  // History for sparklines
  const [cpuHistory, setCpuHistory] = useState<number[]>([]);
  const [memoryHistory, setMemoryHistory] = useState<number[]>([]);
  const [diskHistory, setDiskHistory] = useState<number[]>([]);
  const [networkRxHistory, setNetworkRxHistory] = useState<number[]>([]);
  const [networkTxHistory, setNetworkTxHistory] = useState<number[]>([]);
  
  const intervalRef = useRef<number | null>(null);

  const fetchMetrics = async () => {
    try {
      const data = await invoke<SystemMetrics>("get_system_metrics");
      setMetrics(data);
      setError(null);
      setLoading(false);
      
      // Update history for sparklines
      setCpuHistory(prev => [...prev.slice(-MAX_HISTORY + 1), data.cpu.usage]);
      setMemoryHistory(prev => [...prev.slice(-MAX_HISTORY + 1), data.memory.usage]);
      setDiskHistory(prev => [...prev.slice(-MAX_HISTORY + 1), data.disk.usage]);
      setNetworkRxHistory(prev => [...prev.slice(-MAX_HISTORY + 1), data.network.rxSpeed]);
      setNetworkTxHistory(prev => [...prev.slice(-MAX_HISTORY + 1), data.network.txSpeed]);
    } catch (err) {
      console.error("Failed to fetch metrics:", err);
      setError(err instanceof Error ? err.message : "Failed to load metrics");
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMetrics();
    
    if (!isPaused) {
      intervalRef.current = window.setInterval(fetchMetrics, 1000);
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [isPaused]);

  const handleRefresh = () => {
    fetchMetrics();
  };

  const togglePause = () => {
    setIsPaused(!isPaused);
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatSpeed = (bytesPerSec: number) => {
    if (bytesPerSec < 1024) return `${bytesPerSec.toFixed(0)} B/s`;
    if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(2)} KB/s`;
    if (bytesPerSec < 1024 * 1024 * 1024) return `${(bytesPerSec / 1024 / 1024).toFixed(2)} MB/s`;
    return `${(bytesPerSec / 1024 / 1024 / 1024).toFixed(2)} GB/s`;
  };

  const formatFrequency = (hz: number) => {
    if (hz >= 1_000_000_000) return `${(hz / 1_000_000_000).toFixed(2)} GHz`;
    if (hz >= 1_000_000) return `${(hz / 1_000_000).toFixed(0)} MHz`;
    return `${hz} Hz`;
  };

  const getStatusColor = (usage: number, type: "cpu" | "memory" | "disk") => {
    const thresholds = {
      cpu: { warning: 70, critical: 90 },
      memory: { warning: 80, critical: 95 },
      disk: { warning: 85, critical: 95 },
    };
    const t = thresholds[type];
    if (usage >= t.critical) return "text-neon-red";
    if (usage >= t.warning) return "text-neon-amber";
    return "text-neon-green";
  };

  const getProgressColor = (usage: number, type: "cpu" | "memory" | "disk") => {
    const thresholds = {
      cpu: { warning: 70, critical: 90 },
      memory: { warning: 80, critical: 95 },
      disk: { warning: 85, critical: 95 },
    };
    const t = thresholds[type];
    if (usage >= t.critical) return "bg-neon-red";
    if (usage >= t.warning) return "bg-neon-amber";
    return "bg-neon-green";
  };

  const Sparkline = ({ data, color = "cyan" }: { data: number[]; color?: "cyan" | "green" | "amber" | "red" }) => {
    if (data.length < 2) return null;
    
    const max = Math.max(...data, 1);
    const min = Math.min(...data, 0);
    const range = max - min || 1;
    const width = 200;
    const height = 40;
    const padding = 4;
    
    const points = data.map((value, index) => {
      const x = ((index / (data.length - 1)) * (width - padding * 2)) + padding;
      const y = height - padding - ((value - min) / range) * (height - padding * 2);
      return `${x},${y}`;
    }).join(" ");
    
    const strokeColor = {
      cyan: "#00ffff",
      green: "#00ff88",
      amber: "#ffaa00",
      red: "#ff4444",
    }[color] || "#00ffff";
    
    return (
      <svg width={width} height={height} className="opacity-70">
        <polyline
          points={points}
          fill="none"
          stroke={strokeColor}
          strokeWidth="2"
          className="transition-all"
        />
      </svg>
    );
  };

  const MetricCard = ({ 
    title, 
    icon: Icon, 
    value, 
    unit, 
    subtitle, 
    usage, 
    type, 
    history,
    additionalInfo,
    sparklineColor = "cyan"
  }: {
    title: string;
    icon: typeof Cpu;
    value: string | number;
    unit?: string;
    subtitle?: string;
    usage: number;
    type: "cpu" | "memory" | "disk";
    history: number[];
    additionalInfo?: React.ReactNode;
    sparklineColor?: "cyan" | "green" | "amber" | "red";
  }) => {
    const hasAlert = (type === "cpu" && usage >= 90) || 
                     (type === "memory" && usage >= 95) || 
                     (type === "disk" && usage >= 95);
    
    return (
      <Card className="relative overflow-hidden">
        {hasAlert && (
          <div className="absolute top-2 right-2">
            <AlertTriangle className="w-5 h-5 text-neon-red animate-pulse" />
          </div>
        )}
        <div className="flex items-start justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg ${
              sparklineColor === "cyan" ? "bg-neon-cyan/20" :
              sparklineColor === "green" ? "bg-neon-green/20" :
              sparklineColor === "amber" ? "bg-neon-amber/20" :
              "bg-gray-500/20"
            }`}>
              <Icon className={`w-6 h-6 ${
                sparklineColor === "cyan" ? "text-neon-cyan" :
                sparklineColor === "green" ? "text-neon-green" :
                sparklineColor === "amber" ? "text-neon-amber" :
                "text-gray-400"
              }`} />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-gray-300">{title}</h3>
              {subtitle && <p className="text-xs text-gray-500">{subtitle}</p>}
            </div>
          </div>
        </div>
        
        <div className="mb-3">
          <div className={`text-4xl font-bold ${getStatusColor(usage, type)} transition-all`}>
            {typeof value === "number" ? value.toFixed(1) : value}
            {unit && <span className="text-2xl ml-1">{unit}</span>}
          </div>
          {additionalInfo && (
            <div className="text-sm text-gray-400 mt-1">{additionalInfo}</div>
          )}
        </div>
        
        <div className="mb-3">
          <div className="flex items-center justify-between text-xs text-gray-400 mb-1">
            <span>Usage</span>
            <span className={getStatusColor(usage, type)}>{usage.toFixed(1)}%</span>
          </div>
          <div className="w-full bg-gray-800/50 rounded-full h-3 overflow-hidden">
            <div
              className={`h-3 rounded-full transition-all duration-300 ${getProgressColor(usage, type)}`}
              style={{ width: `${Math.min(usage, 100)}%` }}
            />
          </div>
        </div>
        
        {history.length > 1 && (
          <div className="mt-3 pt-3 border-t border-white/10">
            <Sparkline data={history} color={sparklineColor} />
          </div>
        )}
      </Card>
    );
  };

  if (loading && !metrics) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <RefreshCw className="w-8 h-8 mx-auto mb-4 text-neon-cyan animate-spin" />
          <p className="text-gray-400">Loading system metrics...</p>
        </div>
      </div>
    );
  }

  if (error && !metrics) {
    return (
      <div className="flex items-center justify-center h-64">
        <Card className="max-w-md">
          <div className="text-center">
            <AlertTriangle className="w-12 h-12 mx-auto mb-4 text-neon-red" />
            <h3 className="text-lg font-semibold mb-2">Failed to Load Metrics</h3>
            <p className="text-sm text-gray-400 mb-4">{error}</p>
            <Button onClick={handleRefresh} variant="primary">
              <RefreshCw className="w-4 h-4 mr-2" />
              Retry
            </Button>
          </div>
        </Card>
      </div>
    );
  }

  if (!metrics) return null;

  return (
    <div className="space-y-6">
      {/* Header with Controls */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
            System Monitor Hub
          </h1>
          <p className="text-gray-400">Real-time system performance monitoring</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            onClick={togglePause}
            className="flex items-center gap-2"
          >
            {isPaused ? (
              <>
                <Play className="w-4 h-4" />
                Resume
              </>
            ) : (
              <>
                <Pause className="w-4 h-4" />
                Pause
              </>
            )}
          </Button>
          <Button
            variant="secondary"
            onClick={handleRefresh}
            className="flex items-center gap-2"
          >
            <RefreshCw className={`w-4 h-4 ${!isPaused ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Main Metrics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <MetricCard
          title="CPU"
          icon={Cpu}
          value={metrics.cpu.usage}
          unit="%"
          subtitle={`${metrics.cpu.cores} cores @ ${formatFrequency(metrics.cpu.frequency)}`}
          usage={metrics.cpu.usage}
          type="cpu"
          history={cpuHistory}
          sparklineColor="cyan"
        />
        
        <MetricCard
          title="Memory"
          icon={MemoryStick}
          value={formatBytes(metrics.memory.used)}
          subtitle={`${formatBytes(metrics.memory.free)} free of ${formatBytes(metrics.memory.total)}`}
          usage={metrics.memory.usage}
          type="memory"
          history={memoryHistory}
          additionalInfo={
            <div className="flex items-center gap-4 text-xs">
              <span>Used: {formatBytes(metrics.memory.used)}</span>
              <span>Free: {formatBytes(metrics.memory.free)}</span>
            </div>
          }
          sparklineColor="green"
        />
        
        <MetricCard
          title="Disk"
          icon={HardDrive}
          value={formatBytes(metrics.disk.used)}
          subtitle={`${formatBytes(metrics.disk.free)} free of ${formatBytes(metrics.disk.total)}`}
          usage={metrics.disk.usage}
          type="disk"
          history={diskHistory}
          additionalInfo={
            <div className="flex items-center gap-4 text-xs">
              <span>Used: {formatBytes(metrics.disk.used)}</span>
              <span>Free: {formatBytes(metrics.disk.free)}</span>
            </div>
          }
          sparklineColor="amber"
        />
        
        <Card className="relative overflow-hidden">
          <div className="flex items-start justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-neon-green/20">
                <Wifi className="w-6 h-6 text-neon-green" />
              </div>
              <div>
                <h3 className="text-sm font-semibold text-gray-300">Network</h3>
                <p className="text-xs text-gray-500">Data transfer</p>
              </div>
            </div>
          </div>
          
          <div className="space-y-3">
            <div>
              <div className="flex items-center justify-between mb-1">
                <div className="flex items-center gap-2 text-sm text-gray-400">
                  <TrendingDown className="w-4 h-4 text-neon-green" />
                  <span>Download</span>
                </div>
                <span className="text-lg font-semibold text-neon-green">
                  {formatSpeed(metrics.network.rxSpeed)}
                </span>
              </div>
              {networkRxHistory.length > 1 && (
                <Sparkline data={networkRxHistory} color="green" />
              )}
            </div>
            
            <div className="pt-3 border-t border-white/10">
              <div className="flex items-center justify-between mb-1">
                <div className="flex items-center gap-2 text-sm text-gray-400">
                  <TrendingUp className="w-4 h-4 text-neon-cyan" />
                  <span>Upload</span>
                </div>
                <span className="text-lg font-semibold text-neon-cyan">
                  {formatSpeed(metrics.network.txSpeed)}
                </span>
              </div>
              {networkTxHistory.length > 1 && (
                <Sparkline data={networkTxHistory} color="cyan" />
              )}
            </div>
            
            <div className="pt-3 border-t border-white/10 text-xs text-gray-500">
              <div className="flex justify-between">
                <span>Total RX: {formatBytes(metrics.network.rx)}</span>
                <span>Total TX: {formatBytes(metrics.network.tx)}</span>
              </div>
            </div>
          </div>
        </Card>
      </div>

      {/* Process List */}
      <ProcessList />
    </div>
  );
}
