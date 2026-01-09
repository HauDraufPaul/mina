import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import Tabs from "../../ui/Tabs";
import Modal from "../../ui/Modal";
import { useErrorHandler } from "@/utils/errorHandler";
import { 
  Activity, 
  AlertCircle, 
  CheckCircle, 
  XCircle, 
  Plus, 
  Clock, 
  TrendingUp,
  RefreshCw,
  Search,
  X,
  Zap,
  Server,
  AlertTriangle,
  Info,
  BarChart3,
  Filter
} from "lucide-react";

interface HealthCheck {
  id: number;
  name: string;
  url: string;
  status: string;
  last_check: number;
  response_time?: number;
  error?: string;
}

interface Alert {
  id: number;
  name: string;
  severity: string;
  message: string;
  source: string;
  created_at: number;
  resolved_at?: number;
}

export default function DevOpsControl() {
  const errorHandler = useErrorHandler();
  const [healthChecks, setHealthChecks] = useState<HealthCheck[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [view, setView] = useState<"health" | "alerts" | "metrics">("health");
  const [loading, setLoading] = useState(true);
  const [isPaused, setIsPaused] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [severityFilter, setSeverityFilter] = useState<string>("all");
  
  // Modal states
  const [showHealthCheckModal, setShowHealthCheckModal] = useState(false);
  const [showAlertModal, setShowAlertModal] = useState(false);
  const [newHealthCheckName, setNewHealthCheckName] = useState("");
  const [newHealthCheckUrl, setNewHealthCheckUrl] = useState("");
  const [newAlertName, setNewAlertName] = useState("");
  const [newAlertSeverity, setNewAlertSeverity] = useState("warning");
  const [newAlertMessage, setNewAlertMessage] = useState("");
  const [newAlertSource, setNewAlertSource] = useState("");

  // Response time history for charts
  const [responseTimeHistory, setResponseTimeHistory] = useState<Map<number, number[]>>(new Map());

  useEffect(() => {
    loadData();
    if (!isPaused) {
      const interval = setInterval(loadData, 5000);
      return () => clearInterval(interval);
    }
  }, [isPaused]);

  const loadData = async () => {
    try {
      const [checks, alertsData] = await Promise.all([
        invoke<HealthCheck[]>("list_health_checks"),
        invoke<Alert[]>("list_alerts", { limit: 50, unresolvedOnly: false }),
      ]);
      
      setHealthChecks(checks);
      setAlerts(alertsData);
      
      // Update response time history
      const newHistory = new Map(responseTimeHistory);
      checks.forEach(check => {
        if (check.response_time !== undefined && check.response_time !== null) {
          const existing = newHistory.get(check.id) || [];
          existing.push(check.response_time);
          if (existing.length > 30) existing.shift();
          newHistory.set(check.id, existing);
        }
      });
      setResponseTimeHistory(newHistory);
      
      setLoading(false);
    } catch (error) {
      errorHandler.showError("Failed to load data", error);
      setLoading(false);
    }
  };

  const handleCreateHealthCheck = async () => {
    if (!newHealthCheckName.trim() || !newHealthCheckUrl.trim()) {
      errorHandler.showError("Please enter both name and URL");
      return;
    }

    try {
      new URL(newHealthCheckUrl.trim());
    } catch {
      errorHandler.showError("Please enter a valid URL");
      return;
    }

    if (!/^[a-zA-Z0-9_\s-]+$/.test(newHealthCheckName.trim())) {
      errorHandler.showError("Name can only contain letters, numbers, underscores, hyphens, and spaces");
      return;
    }

    try {
      await invoke("create_health_check", {
        name: newHealthCheckName.trim(),
        url: newHealthCheckUrl.trim(),
      });
      setNewHealthCheckName("");
      setNewHealthCheckUrl("");
      setShowHealthCheckModal(false);
      await loadData();
      errorHandler.showSuccess("Health check created successfully");
    } catch (error) {
      errorHandler.showError("Failed to create health check", error);
    }
  };

  const handleCreateAlert = async () => {
    if (!newAlertName.trim() || !newAlertMessage.trim() || !newAlertSource.trim()) {
      errorHandler.showError("Please fill in all required fields");
      return;
    }

    try {
      await invoke("create_alert", {
        name: newAlertName.trim(),
        severity: newAlertSeverity,
        message: newAlertMessage.trim(),
        source: newAlertSource.trim(),
      });
      setNewAlertName("");
      setNewAlertMessage("");
      setNewAlertSource("");
      setNewAlertSeverity("warning");
      setShowAlertModal(false);
      await loadData();
      errorHandler.showSuccess("Alert created successfully");
    } catch (error) {
      errorHandler.showError("Failed to create alert", error);
    }
  };

  const handleResolveAlert = async (id: number) => {
    try {
      await invoke("resolve_alert", { id });
      await loadData();
      errorHandler.showSuccess("Alert resolved successfully");
    } catch (error) {
      errorHandler.showError("Failed to resolve alert", error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case "healthy":
        return <CheckCircle className="w-5 h-5 text-neon-green" />;
      case "unhealthy":
        return <XCircle className="w-5 h-5 text-neon-red" />;
      default:
        return <Clock className="w-5 h-5 text-neon-amber" />;
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity.toLowerCase()) {
      case "critical":
        return <XCircle className="w-5 h-5 text-neon-red" />;
      case "warning":
        return <AlertTriangle className="w-5 h-5 text-neon-amber" />;
      default:
        return <Info className="w-5 h-5 text-neon-cyan" />;
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity.toLowerCase()) {
      case "critical":
        return "text-neon-red bg-neon-red/20 border-neon-red/50";
      case "warning":
        return "text-neon-amber bg-neon-amber/20 border-neon-amber/50";
      default:
        return "text-neon-cyan bg-neon-cyan/20 border-neon-cyan/50";
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const formatTimeAgo = (timestamp: number) => {
    const seconds = Math.floor((Date.now() / 1000 - timestamp));
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
  };

  // Filtered data
  const filteredAlerts = useMemo(() => {
    let filtered = alerts;
    
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(alert => 
        alert.name.toLowerCase().includes(query) ||
        alert.message.toLowerCase().includes(query) ||
        alert.source.toLowerCase().includes(query)
      );
    }
    
    if (severityFilter !== "all") {
      filtered = filtered.filter(alert => alert.severity.toLowerCase() === severityFilter.toLowerCase());
    }
    
    return filtered;
  }, [alerts, searchQuery, severityFilter]);

  const healthyCount = healthChecks.filter(c => c.status === "healthy").length;
  const unhealthyCount = healthChecks.filter(c => c.status === "unhealthy").length;
  const unknownCount = healthChecks.filter(c => c.status === "unknown").length;
  const activeAlertsCount = alerts.filter(a => !a.resolved_at).length;
  const criticalAlertsCount = alerts.filter(a => !a.resolved_at && a.severity === "critical").length;

  // Response time sparkline component
  const ResponseTimeSparkline = ({ data, color }: { data: number[]; color: string }) => {
    if (data.length < 2) return null;
    
    const max = Math.max(...data, 1);
    const min = Math.min(...data, 0);
    const range = max - min || 1;
    const width = 120;
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
          <Activity className="w-8 h-8 mx-auto mb-4 text-neon-cyan animate-spin" />
          <p className="text-gray-400">Loading DevOps data...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
            DevOps Control
          </h1>
          <p className="text-gray-400">Health monitoring, alerts, and Prometheus metrics</p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            onClick={() => setIsPaused(!isPaused)}
            className="flex items-center gap-2"
          >
            {isPaused ? (
              <>
                <RefreshCw className="w-4 h-4" />
                Resume
              </>
            ) : (
              <>
                <Clock className="w-4 h-4" />
                Pause
              </>
            )}
          </Button>
          <Button
            variant="secondary"
            onClick={loadData}
            className="flex items-center gap-2"
          >
            <RefreshCw className={`w-4 h-4 ${!isPaused ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Overview Stats */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <Card className="bg-gradient-to-br from-neon-green/20 to-transparent border-neon-green/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-green/20">
              <CheckCircle className="w-5 h-5 text-neon-green" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Healthy</div>
              <div className="text-2xl font-bold text-neon-green">{healthyCount}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-neon-red/20 to-transparent border-neon-red/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-red/20">
              <XCircle className="w-5 h-5 text-neon-red" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Unhealthy</div>
              <div className="text-2xl font-bold text-neon-red">{unhealthyCount}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-neon-amber/20 to-transparent border-neon-amber/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-amber/20">
              <Clock className="w-5 h-5 text-neon-amber" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Unknown</div>
              <div className="text-2xl font-bold text-neon-amber">{unknownCount}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-purple-500/20 to-transparent border-purple-500/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-purple-500/20">
              <AlertCircle className="w-5 h-5 text-purple-400" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Active Alerts</div>
              <div className="text-2xl font-bold text-purple-400">{activeAlertsCount}</div>
            </div>
          </div>
        </Card>
        
        <Card className="bg-gradient-to-br from-red-600/20 to-transparent border-red-600/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-red-600/20">
              <Zap className="w-5 h-5 text-red-400" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Critical</div>
              <div className="text-2xl font-bold text-red-400">{criticalAlertsCount}</div>
            </div>
          </div>
        </Card>
      </div>

      {/* View Toggle */}
      <Tabs
        items={[
          {
            id: "health",
            label: `Health Checks (${healthChecks.length})`,
            icon: <Activity className="w-4 h-4" />,
          },
          {
            id: "alerts",
            label: `Alerts (${activeAlertsCount})`,
            icon: <AlertCircle className="w-4 h-4" />,
          },
          {
            id: "metrics",
            label: "Metrics",
            icon: <TrendingUp className="w-4 h-4" />,
          },
        ]}
        activeTab={view}
        onTabChange={(tabId) => setView(tabId as "health" | "alerts" | "metrics")}
      />

      {/* Health Checks View */}
      {view === "health" && (
        <>
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-bold">Health Checks</h2>
            <Button
              variant="primary"
              onClick={() => setShowHealthCheckModal(true)}
            >
              <Plus className="w-4 h-4 mr-2" />
              New Health Check
            </Button>
          </div>

          {healthChecks.length === 0 ? (
            <Card>
              <div className="text-center py-12 text-gray-400">
                <Server className="w-16 h-16 mx-auto mb-4 text-gray-500" />
                <p className="text-lg mb-2">No health checks configured</p>
                <p className="text-sm mb-4">Create your first health check to start monitoring services</p>
                <Button
                  variant="primary"
                  onClick={() => setShowHealthCheckModal(true)}
                >
                  <Plus className="w-4 h-4 mr-2" />
                  Create Health Check
                </Button>
              </div>
            </Card>
          ) : (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              {healthChecks.map((check) => {
                const history = responseTimeHistory.get(check.id) || [];
                const avgResponseTime = history.length > 0
                  ? Math.round(history.reduce((a, b) => a + b, 0) / history.length)
                  : check.response_time || 0;
                
                return (
                  <Card
                    key={check.id}
                    className={`relative overflow-hidden transition-all hover:border-opacity-100 ${
                      check.status === "healthy"
                        ? "border-neon-green/30"
                        : check.status === "unhealthy"
                        ? "border-neon-red/30"
                        : "border-neon-amber/30"
                    }`}
                  >
                    {check.status === "healthy" && (
                      <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-neon-green via-neon-cyan to-neon-green" />
                    )}
                    {check.status === "unhealthy" && (
                      <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-neon-red via-red-600 to-neon-red" />
                    )}
                    
                    <div className="flex items-start justify-between mb-4">
                      <div className="flex items-center gap-3">
                        {getStatusIcon(check.status)}
                        <div>
                          <h3 className="font-semibold text-lg text-gray-200">{check.name}</h3>
                          <p className="text-sm text-gray-400 font-mono">{check.url}</p>
                        </div>
                      </div>
                      <span
                        className={`text-xs px-2 py-1 rounded font-semibold ${
                          check.status === "healthy"
                            ? "bg-neon-green/20 text-neon-green"
                            : check.status === "unhealthy"
                            ? "bg-neon-red/20 text-neon-red"
                            : "bg-neon-amber/20 text-neon-amber"
                        }`}
                      >
                        {check.status.toUpperCase()}
                      </span>
                    </div>

                    <div className="space-y-3">
                      {check.response_time !== undefined && check.response_time !== null && (
                        <div>
                          <div className="flex items-center justify-between text-xs text-gray-400 mb-1">
                            <span>Response Time</span>
                            <span className="font-semibold text-gray-300">
                              {check.response_time}ms {avgResponseTime > 0 && `(avg: ${avgResponseTime}ms)`}
                            </span>
                          </div>
                          {history.length > 1 && (
                            <ResponseTimeSparkline
                              data={history}
                              color={
                                check.status === "healthy" ? "#00ff88" :
                                check.status === "unhealthy" ? "#ff4444" : "#ffb000"
                              }
                            />
                          )}
                        </div>
                      )}

                      {check.error && (
                        <div className="p-2 glass-card rounded border border-neon-red/30">
                          <p className="text-xs text-neon-red">{check.error}</p>
                        </div>
                      )}

                      <div className="pt-2 border-t border-white/10">
                        <div className="flex items-center justify-between text-xs text-gray-500">
                          <span className="flex items-center gap-1">
                            <Clock className="w-3 h-3" />
                            Last check: {formatTimeAgo(check.last_check)}
                          </span>
                          <span>{formatTime(check.last_check)}</span>
                        </div>
                      </div>
                    </div>
                  </Card>
                );
              })}
            </div>
          )}
        </>
      )}

      {/* Alerts View */}
      {view === "alerts" && (
        <>
          <div className="flex items-center justify-between">
            <h2 className="text-2xl font-bold">Alerts</h2>
            <Button
              variant="primary"
              onClick={() => setShowAlertModal(true)}
            >
              <Plus className="w-4 h-4 mr-2" />
              New Alert
            </Button>
          </div>

          {/* Filters */}
          <Card>
            <div className="flex items-center gap-4">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search alerts..."
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
              <div className="flex items-center gap-2">
                <Filter className="w-4 h-4 text-gray-400" />
                <select
                  value={severityFilter}
                  onChange={(e) => setSeverityFilter(e.target.value)}
                  className="glass-input"
                >
                  <option value="all">All Severities</option>
                  <option value="critical">Critical</option>
                  <option value="warning">Warning</option>
                  <option value="info">Info</option>
                </select>
              </div>
            </div>
          </Card>

          {filteredAlerts.length === 0 ? (
            <Card>
              <div className="text-center py-12 text-gray-400">
                <CheckCircle className="w-16 h-16 mx-auto mb-4 text-neon-green" />
                <p className="text-lg mb-2">No alerts found</p>
                <p className="text-sm">
                  {searchQuery || severityFilter !== "all"
                    ? "Try adjusting your filters"
                    : "All systems operational"}
                </p>
              </div>
            </Card>
          ) : (
            <div className="space-y-3">
              {filteredAlerts.map((alert) => (
                <Card
                  key={alert.id}
                  className={`relative overflow-hidden border-l-4 ${
                    alert.severity === "critical"
                      ? "border-neon-red"
                      : alert.severity === "warning"
                      ? "border-neon-amber"
                      : "border-neon-cyan"
                  } ${alert.resolved_at ? "opacity-60" : ""}`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        {getSeverityIcon(alert.severity)}
                        <span className={`font-semibold text-lg ${getSeverityColor(alert.severity).split(" ")[0]}`}>
                          {alert.name}
                        </span>
                        <span className={`text-xs px-2 py-1 rounded ${getSeverityColor(alert.severity)}`}>
                          {alert.severity.toUpperCase()}
                        </span>
                        {alert.resolved_at && (
                          <span className="text-xs px-2 py-1 rounded bg-gray-500/20 text-gray-400">
                            RESOLVED
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-300 mb-3">{alert.message}</p>
                      <div className="flex items-center gap-4 text-xs text-gray-500">
                        <span className="flex items-center gap-1">
                          <Server className="w-3 h-3" />
                          {alert.source}
                        </span>
                        <span className="flex items-center gap-1">
                          <Clock className="w-3 h-3" />
                          {formatTimeAgo(alert.created_at)}
                        </span>
                        <span>{formatTime(alert.created_at)}</span>
                      </div>
                    </div>
                    {!alert.resolved_at && (
                      <Button
                        variant="secondary"
                        onClick={() => handleResolveAlert(alert.id)}
                        className="ml-4"
                      >
                        Resolve
                      </Button>
                    )}
                  </div>
                </Card>
              ))}
            </div>
          )}
        </>
      )}

      {/* Metrics View */}
      {view === "metrics" && (
        <Card title="Prometheus Metrics">
          <div className="text-center py-12 text-gray-400">
            <BarChart3 className="w-16 h-16 mx-auto mb-4 text-gray-500" />
            <p className="text-lg mb-2">Metrics Visualization</p>
            <p className="text-sm mb-4">
              Use the backend commands to save and retrieve Prometheus metrics
            </p>
            <div className="mt-6 p-4 glass-card rounded text-left max-w-2xl mx-auto">
              <p className="text-sm font-mono text-gray-300 mb-2">Available Commands:</p>
              <ul className="text-xs text-gray-400 space-y-1">
                <li>• <code className="text-neon-cyan">save_prometheus_metric</code> - Save a metric</li>
                <li>• <code className="text-neon-cyan">get_prometheus_metrics</code> - Retrieve metrics by name and time range</li>
              </ul>
            </div>
          </div>
        </Card>
      )}

      {/* Create Health Check Modal */}
      <Modal
        isOpen={showHealthCheckModal}
        onClose={() => {
          setShowHealthCheckModal(false);
          setNewHealthCheckName("");
          setNewHealthCheckUrl("");
        }}
        title="Create Health Check"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Service Name
            </label>
            <input
              type="text"
              value={newHealthCheckName}
              onChange={(e) => setNewHealthCheckName(e.target.value)}
              onKeyPress={(e) => {
                if (e.key === "Enter") {
                  handleCreateHealthCheck();
                }
              }}
              className="glass-input w-full"
              placeholder="e.g., API Server, Database..."
              autoFocus
            />
            <p className="text-xs text-gray-500 mt-1">
              Letters, numbers, underscores, hyphens, and spaces only
            </p>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Health Check URL
            </label>
            <input
              type="url"
              value={newHealthCheckUrl}
              onChange={(e) => setNewHealthCheckUrl(e.target.value)}
              onKeyPress={(e) => {
                if (e.key === "Enter") {
                  handleCreateHealthCheck();
                }
              }}
              className="glass-input w-full font-mono"
              placeholder="https://example.com/health"
            />
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowHealthCheckModal(false);
                setNewHealthCheckName("");
                setNewHealthCheckUrl("");
              }}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleCreateHealthCheck}
              disabled={!newHealthCheckName.trim() || !newHealthCheckUrl.trim()}
            >
              Create
            </Button>
          </div>
        </div>
      </Modal>

      {/* Create Alert Modal */}
      <Modal
        isOpen={showAlertModal}
        onClose={() => {
          setShowAlertModal(false);
          setNewAlertName("");
          setNewAlertMessage("");
          setNewAlertSource("");
          setNewAlertSeverity("warning");
        }}
        title="Create Alert"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Alert Name
            </label>
            <input
              type="text"
              value={newAlertName}
              onChange={(e) => setNewAlertName(e.target.value)}
              className="glass-input w-full"
              placeholder="e.g., High CPU Usage, Database Connection Failed..."
              autoFocus
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Severity
            </label>
            <select
              value={newAlertSeverity}
              onChange={(e) => setNewAlertSeverity(e.target.value)}
              className="glass-input w-full"
            >
              <option value="info">Info</option>
              <option value="warning">Warning</option>
              <option value="critical">Critical</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Message
            </label>
            <textarea
              value={newAlertMessage}
              onChange={(e) => setNewAlertMessage(e.target.value)}
              className="glass-input w-full min-h-[100px]"
              placeholder="Detailed alert message..."
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Source
            </label>
            <input
              type="text"
              value={newAlertSource}
              onChange={(e) => setNewAlertSource(e.target.value)}
              className="glass-input w-full"
              placeholder="e.g., System Monitor, API Gateway..."
            />
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowAlertModal(false);
                setNewAlertName("");
                setNewAlertMessage("");
                setNewAlertSource("");
                setNewAlertSeverity("warning");
              }}
            >
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleCreateAlert}
              disabled={!newAlertName.trim() || !newAlertMessage.trim() || !newAlertSource.trim()}
            >
              Create Alert
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
