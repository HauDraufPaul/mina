import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { 
  Wifi, 
  Globe, 
  RefreshCw, 
  Search, 
  X, 
  Network,
  Activity,
  Pause,
  Play,
  Download,
  Upload
} from "lucide-react";

interface NetworkInterface {
  name: string;
  received: number;
  transmitted: number;
}

interface NetworkConnection {
  local_address: string;
  remote_address: string;
  protocol: string;
  state: string;
  process_id?: number;
}

export default function NetworkConstellation() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [connections, setConnections] = useState<NetworkConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedView, setSelectedView] = useState<"overview" | "interfaces" | "connections">("overview");
  const [isPaused, setIsPaused] = useState(false);
  const [selectedInterface, setSelectedInterface] = useState<string | null>(null);
  
  // History for traffic visualization
  const [trafficHistory, setTrafficHistory] = useState<Map<string, { rx: number[]; tx: number[] }>>(new Map());

  const fetchData = async () => {
    try {
      const [interfacesData, connectionsData] = await Promise.all([
        invoke<NetworkInterface[]>("get_network_interfaces"),
        invoke<NetworkConnection[]>("get_network_connections"),
      ]);
      
      setInterfaces(interfacesData);
      setConnections(connectionsData);
      setError(null);
      setLoading(false);
      
      // Update traffic history
      const newHistory = new Map(trafficHistory);
      interfacesData.forEach(iface => {
        const existing = newHistory.get(iface.name) || { rx: [], tx: [] };
        existing.rx.push(iface.received);
        existing.tx.push(iface.transmitted);
        if (existing.rx.length > 60) existing.rx.shift();
        if (existing.tx.length > 60) existing.tx.shift();
        newHistory.set(iface.name, existing);
      });
      setTrafficHistory(newHistory);
    } catch (err) {
      console.error("Failed to fetch network data:", err);
      setError(err instanceof Error ? err.message : "Failed to load network data");
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    
    if (!isPaused) {
      const interval = setInterval(fetchData, 2000);
      return () => clearInterval(interval);
    }
  }, [isPaused]);

  // Filter interfaces based on search
  const filteredInterfaces = useMemo(() => {
    if (!searchQuery.trim()) return interfaces;
    const query = searchQuery.toLowerCase();
    return interfaces.filter(iface => 
      iface.name.toLowerCase().includes(query)
    );
  }, [interfaces, searchQuery]);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const getTotalTraffic = (iface: NetworkInterface) => {
    return iface.received + iface.transmitted;
  };

  const getTrafficColor = (iface: NetworkInterface) => {
    const total = getTotalTraffic(iface);
    const maxTotal = Math.max(...interfaces.map(getTotalTraffic), 1);
    const ratio = total / maxTotal;
    
    if (ratio > 0.7) return "text-neon-red";
    if (ratio > 0.4) return "text-neon-amber";
    if (ratio > 0.1) return "text-neon-green";
    return "text-neon-cyan";
  };

  const getConnectionColor = (state: string) => {
    switch (state.toLowerCase()) {
      case "established":
        return "#00ff88";
      case "listening":
        return "#00d9ff";
      case "time_wait":
        return "#ffb000";
      case "closed":
        return "#666666";
      default:
        return "#ffffff";
    }
  };

  const Sparkline = ({ data, color }: { data: number[]; color: string }) => {
    if (data.length < 2) return null;
    
    const max = Math.max(...data, 1);
    const min = Math.min(...data, 0);
    const range = max - min || 1;
    const width = 150;
    const height = 30;
    const padding = 2;
    
    const points = data.map((value, index) => {
      const x = ((index / (data.length - 1)) * (width - padding * 2)) + padding;
      const y = height - padding - ((value - min) / range) * (height - padding * 2);
      return `${x},${y}`;
    }).join(" ");
    
    return (
      <svg width={width} height={height} className="opacity-70">
        <polyline
          points={points}
          fill="none"
          stroke={color}
          strokeWidth="1.5"
          className="transition-all"
        />
      </svg>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <Network className="w-8 h-8 mx-auto mb-4 text-neon-cyan animate-spin" />
          <p className="text-gray-400">Loading network data...</p>
        </div>
      </div>
    );
  }

  if (error && interfaces.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <Card className="max-w-md">
          <div className="text-center">
            <Network className="w-12 h-12 mx-auto mb-4 text-neon-red" />
            <h3 className="text-lg font-semibold mb-2">Connection Error</h3>
            <p className="text-sm text-gray-400 mb-4">{error}</p>
            <Button onClick={fetchData} variant="primary">
              <RefreshCw className="w-4 h-4 mr-2" />
              Retry
            </Button>
          </div>
        </Card>
      </div>
    );
  }

  const totalReceived = interfaces.reduce((sum, i) => sum + i.received, 0);
  const totalTransmitted = interfaces.reduce((sum, i) => sum + i.transmitted, 0);
  const activeConnections = connections.filter(c => c.state.toLowerCase() === "established").length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
            Network Monitor
          </h1>
          <p className="text-gray-400">Real-time network interface and connection monitoring</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            onClick={() => setIsPaused(!isPaused)}
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
            onClick={fetchData}
            className="flex items-center gap-2"
          >
            <RefreshCw className={`w-4 h-4 ${!isPaused ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Overview Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="bg-gradient-to-br from-neon-cyan/20 to-transparent border-neon-cyan/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-cyan/20">
              <Network className="w-5 h-5 text-neon-cyan" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Active Interfaces</div>
              <div className="text-2xl font-bold text-neon-cyan">{interfaces.length}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-neon-green/20 to-transparent border-neon-green/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-green/20">
              <Download className="w-5 h-5 text-neon-green" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Total Received</div>
              <div className="text-xl font-bold text-neon-green">{formatBytes(totalReceived)}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-neon-amber/20 to-transparent border-neon-amber/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-amber/20">
              <Upload className="w-5 h-5 text-neon-amber" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Total Transmitted</div>
              <div className="text-xl font-bold text-neon-amber">{formatBytes(totalTransmitted)}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-purple-500/20 to-transparent border-purple-500/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-purple-500/20">
              <Activity className="w-5 h-5 text-purple-400" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Active Connections</div>
              <div className="text-xl font-bold text-purple-400">{activeConnections}</div>
            </div>
          </div>
        </Card>
      </div>

      {/* View Toggle */}
      <div className="flex items-center gap-2">
        <Button
          variant={selectedView === "overview" ? "primary" : "secondary"}
          onClick={() => setSelectedView("overview")}
        >
          <Activity className="w-4 h-4 mr-2" />
          Overview
        </Button>
        <Button
          variant={selectedView === "interfaces" ? "primary" : "secondary"}
          onClick={() => setSelectedView("interfaces")}
        >
          <Wifi className="w-4 h-4 mr-2" />
          Interfaces ({interfaces.length})
        </Button>
        <Button
          variant={selectedView === "connections" ? "primary" : "secondary"}
          onClick={() => setSelectedView("connections")}
        >
          <Globe className="w-4 h-4 mr-2" />
          Connections ({connections.length})
        </Button>
      </div>

      {/* Overview View */}
      {selectedView === "overview" && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Top Interfaces by Traffic */}
          <Card title="Top Interfaces by Traffic" subtitle="Interfaces with highest network activity">
            <div className="space-y-3">
              {[...interfaces]
                .sort((a, b) => getTotalTraffic(b) - getTotalTraffic(a))
                .slice(0, 5)
                .map((iface) => {
                  const total = getTotalTraffic(iface);
                  const maxTotal = Math.max(...interfaces.map(getTotalTraffic), 1);
                  const percentage = (total / maxTotal) * 100;
                  
                  return (
                    <div key={iface.name} className="space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <span className="font-medium text-gray-200">{iface.name}</span>
                        <span className={`font-semibold ${getTrafficColor(iface)}`}>
                          {formatBytes(total)}
                        </span>
                      </div>
                      <div className="h-2 bg-white/5 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-gradient-to-r from-neon-cyan to-neon-green transition-all"
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                      <div className="flex items-center justify-between text-xs text-gray-400">
                        <span className="flex items-center gap-1">
                          <Download className="w-3 h-3 text-neon-green" />
                          {formatBytes(iface.received)}
                        </span>
                        <span className="flex items-center gap-1">
                          <Upload className="w-3 h-3 text-neon-amber" />
                          {formatBytes(iface.transmitted)}
                        </span>
                      </div>
                    </div>
                  );
                })}
            </div>
          </Card>

          {/* Connection States */}
          <Card title="Connection States" subtitle="Breakdown of connection statuses">
            <div className="space-y-3">
              {Object.entries(
                connections.reduce((acc, conn) => {
                  acc[conn.state] = (acc[conn.state] || 0) + 1;
                  return acc;
                }, {} as Record<string, number>)
              )
                .sort((a, b) => b[1] - a[1])
                .map(([state, count]) => (
                  <div key={state} className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div
                        className="w-3 h-3 rounded-full"
                        style={{ backgroundColor: getConnectionColor(state) }}
                      />
                      <span className="text-sm text-gray-300 capitalize">{state.replace(/_/g, " ")}</span>
                    </div>
                    <span className="text-sm font-semibold text-gray-200">{count}</span>
                  </div>
                ))}
            </div>
          </Card>
        </div>
      )}

      {/* Interfaces View */}
      {selectedView === "interfaces" && (
        <>
          <div className="flex items-center gap-4">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search interfaces..."
                className="glass-input w-full pl-10 pr-10"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery("")}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white"
                >
                  <X className="w-4 h-4" />
                </button>
              )}
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredInterfaces.map((iface) => {
              const history = trafficHistory.get(iface.name);
              const total = getTotalTraffic(iface);
              const maxTotal = Math.max(...interfaces.map(getTotalTraffic), 1);
              const percentage = (total / maxTotal) * 100;
              
              return (
                <div
                  key={iface.name}
                  onClick={() => setSelectedInterface(iface.name === selectedInterface ? null : iface.name)}
                  className="cursor-pointer"
                >
                  <Card
                    className={`relative overflow-hidden hover:border-neon-cyan/50 transition-all ${
                      selectedInterface === iface.name ? "border-neon-cyan/50" : ""
                    }`}
                  >
                    {selectedInterface === iface.name && (
                      <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-neon-cyan via-neon-green to-neon-cyan" />
                    )}
                    
                    <div className="flex items-start justify-between mb-4">
                      <div className="flex items-center gap-3">
                        <div className="p-2 rounded-lg bg-neon-cyan/20">
                          <Wifi className="w-5 h-5 text-neon-cyan" />
                        </div>
                        <div>
                          <h3 className="font-semibold text-gray-200">{iface.name}</h3>
                          <p className="text-xs text-gray-500">Network Interface</p>
                        </div>
                      </div>
                      <div className={`text-xs font-semibold ${getTrafficColor(iface)}`}>
                        {percentage.toFixed(1)}%
                      </div>
                    </div>

                    <div className="space-y-3">
                      <div>
                        <div className="flex items-center justify-between text-xs text-gray-400 mb-1">
                          <span className="flex items-center gap-1">
                            <Download className="w-3 h-3 text-neon-green" />
                            Received
                          </span>
                          <span className="text-neon-green font-semibold">
                            {formatBytes(iface.received)}
                          </span>
                        </div>
                        {history && history.rx.length > 1 && (
                          <Sparkline data={history.rx.slice(-20)} color="#00ff88" />
                        )}
                      </div>

                      <div>
                        <div className="flex items-center justify-between text-xs text-gray-400 mb-1">
                          <span className="flex items-center gap-1">
                            <Upload className="w-3 h-3 text-neon-amber" />
                            Transmitted
                          </span>
                          <span className="text-neon-amber font-semibold">
                            {formatBytes(iface.transmitted)}
                          </span>
                        </div>
                        {history && history.tx.length > 1 && (
                          <Sparkline data={history.tx.slice(-20)} color="#ffb000" />
                        )}
                      </div>

                      <div className="pt-2 border-t border-white/10">
                        <div className="flex justify-between text-xs mb-2">
                          <span className="text-gray-400">Total Traffic:</span>
                          <span className="font-semibold text-gray-200">{formatBytes(total)}</span>
                        </div>
                        <div className="h-1.5 bg-white/5 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-gradient-to-r from-neon-cyan to-neon-green transition-all"
                            style={{ width: `${percentage}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  </Card>
                </div>
              );
            })}
          </div>

          {filteredInterfaces.length === 0 && (
            <Card>
              <div className="text-center py-8 text-gray-400">
                <Wifi className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>No interfaces found matching your search</p>
              </div>
            </Card>
          )}
        </>
      )}

      {/* Connections View */}
      {selectedView === "connections" && (
        <Card 
          title="Active Connections" 
          subtitle={`${connections.length} network connections detected`}
        >
          {connections.length === 0 ? (
            <div className="text-center py-8 text-gray-400">
              <Globe className="w-12 h-12 mx-auto mb-4 text-gray-500" />
              <p>No active connections detected</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-white/10">
                    <th className="text-left p-3 text-xs font-semibold text-gray-400">Local Address</th>
                    <th className="text-left p-3 text-xs font-semibold text-gray-400">Remote Address</th>
                    <th className="text-left p-3 text-xs font-semibold text-gray-400">Protocol</th>
                    <th className="text-left p-3 text-xs font-semibold text-gray-400">State</th>
                    <th className="text-left p-3 text-xs font-semibold text-gray-400">Process ID</th>
                  </tr>
                </thead>
                <tbody>
                  {connections.map((conn, index) => (
                    <tr
                      key={index}
                      className="border-b border-white/5 hover:bg-white/5 transition-colors"
                    >
                      <td className="p-3 text-sm font-mono text-gray-300">{conn.local_address}</td>
                      <td className="p-3 text-sm font-mono text-gray-300">{conn.remote_address}</td>
                      <td className="p-3 text-sm text-gray-400">{conn.protocol}</td>
                      <td className="p-3 text-sm">
                        <span
                          className="px-2 py-1 rounded text-xs"
                          style={{
                            backgroundColor: getConnectionColor(conn.state) + "20",
                            color: getConnectionColor(conn.state),
                          }}
                        >
                          {conn.state}
                        </span>
                      </td>
                      <td className="p-3 text-sm font-mono text-gray-400">
                        {conn.process_id || "N/A"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>
      )}
    </div>
  );
}
