import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Activity, AlertCircle, CheckCircle, XCircle, Plus, Clock, TrendingUp } from "lucide-react";

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
  const [healthChecks, setHealthChecks] = useState<HealthCheck[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [view, setView] = useState<"health" | "alerts" | "metrics">("health");
  const [loading, setLoading] = useState(true);
  const [newHealthCheckName, setNewHealthCheckName] = useState("");
  const [newHealthCheckUrl, setNewHealthCheckUrl] = useState("");

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadData = async () => {
    try {
      const [checks, alertsData] = await Promise.all([
        invoke<HealthCheck[]>("list_health_checks"),
        invoke<Alert[]>("list_alerts", { limit: 20, unresolvedOnly: true }),
      ]);
      setHealthChecks(checks);
      setAlerts(alertsData);
      setLoading(false);
    } catch (error) {
      console.error("Failed to load data:", error);
      setLoading(false);
    }
  };

  const handleCreateHealthCheck = async () => {
    if (!newHealthCheckName.trim() || !newHealthCheckUrl.trim()) {
      // TODO: Replace with proper error UI component
      alert("Please enter both name and URL");
      return;
    }

    // Validate URL format
    try {
      new URL(newHealthCheckUrl.trim());
    } catch {
      alert("Please enter a valid URL");
      return;
    }

    // Validate name (alphanumeric, underscore, hyphen, space)
    if (!/^[a-zA-Z0-9_\s-]+$/.test(newHealthCheckName.trim())) {
      alert("Name can only contain letters, numbers, underscores, hyphens, and spaces");
      return;
    }

    try {
      await invoke("create_health_check", {
        name: newHealthCheckName.trim(),
        url: newHealthCheckUrl.trim(),
      });
      setNewHealthCheckName("");
      setNewHealthCheckUrl("");
      await loadData();
    } catch (error) {
      // TODO: Replace with proper error UI component
      console.error("Failed to create health check:", error);
      alert(`Failed to create health check: ${error}`);
    }
  };

  const handleResolveAlert = async (id: number) => {
    try {
      await invoke("resolve_alert", { id });
      await loadData();
    } catch (error) {
      alert(`Failed to resolve alert: ${error}`);
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

  const getSeverityColor = (severity: string) => {
    switch (severity.toLowerCase()) {
      case "critical":
        return "text-neon-red";
      case "warning":
        return "text-neon-amber";
      default:
        return "text-neon-cyan";
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  if (loading) {
    return <div className="text-center">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            DevOps Control
          </h1>
          <p className="text-gray-400">Prometheus and service monitoring</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant={view === "health" ? "primary" : "secondary"}
            onClick={() => setView("health")}
          >
            <Activity className="w-4 h-4 mr-2" />
            Health Checks
          </Button>
          <Button
            variant={view === "alerts" ? "primary" : "secondary"}
            onClick={() => setView("alerts")}
          >
            <AlertCircle className="w-4 h-4 mr-2" />
            Alerts
          </Button>
          <Button
            variant={view === "metrics" ? "primary" : "secondary"}
            onClick={() => setView("metrics")}
          >
            <TrendingUp className="w-4 h-4 mr-2" />
            Metrics
          </Button>
        </div>
      </div>

      {view === "health" && (
        <>
          <Card title="Create Health Check">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Name</label>
                <input
                  type="text"
                  value={newHealthCheckName}
                  onChange={(e) => setNewHealthCheckName(e.target.value)}
                  className="glass-input w-full"
                  placeholder="Service Name"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">URL</label>
                <input
                  type="text"
                  value={newHealthCheckUrl}
                  onChange={(e) => setNewHealthCheckUrl(e.target.value)}
                  className="glass-input w-full"
                  placeholder="https://example.com/health"
                />
              </div>
            </div>
            <Button variant="primary" className="mt-4" onClick={handleCreateHealthCheck}>
              <Plus className="w-4 h-4 mr-2" />
              Create Health Check
            </Button>
          </Card>

          <Card title="Health Checks">
            <div className="space-y-3">
              {healthChecks.length === 0 ? (
                <div className="text-center text-gray-400 py-8">
                  <Activity className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                  <p>No health checks configured</p>
                </div>
              ) : (
                healthChecks.map((check) => (
                  <div key={check.id} className="glass-card p-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        {getStatusIcon(check.status)}
                        <div>
                          <div className="font-semibold">{check.name}</div>
                          <div className="text-sm text-gray-400">{check.url}</div>
                          <div className="text-xs text-gray-500 mt-1">
                            Last check: {formatTime(check.last_check)}
                            {check.response_time && (
                              <span> • {check.response_time}ms</span>
                            )}
                          </div>
                          {check.error && (
                            <div className="text-xs text-neon-red mt-1">{check.error}</div>
                          )}
                        </div>
                      </div>
                      <span
                        className={`text-xs px-2 py-1 rounded ${
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
                  </div>
                ))
              )}
            </div>
          </Card>
        </>
      )}

      {view === "alerts" && (
        <Card title="Active Alerts">
          <div className="space-y-3">
            {alerts.length === 0 ? (
              <div className="text-center text-gray-400 py-8">
                <CheckCircle className="w-12 h-12 mx-auto mb-4 text-neon-green" />
                <p>No active alerts</p>
              </div>
            ) : (
              alerts.map((alert) => (
                <div key={alert.id} className="glass-card p-4 border-l-4 border-neon-red">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <AlertCircle className={`w-5 h-5 ${getSeverityColor(alert.severity)}`} />
                        <span className={`font-semibold ${getSeverityColor(alert.severity)}`}>
                          {alert.name}
                        </span>
                        <span className="text-xs text-gray-400">({alert.severity})</span>
                      </div>
                      <p className="text-sm text-gray-300 mb-2">{alert.message}</p>
                      <div className="text-xs text-gray-500">
                        Source: {alert.source} • {formatTime(alert.created_at)}
                      </div>
                    </div>
                    <Button
                      variant="secondary"
                      onClick={() => handleResolveAlert(alert.id)}
                    >
                      Resolve
                    </Button>
                  </div>
                </div>
              ))
            )}
          </div>
        </Card>
      )}

      {view === "metrics" && (
        <Card title="Prometheus Metrics">
          <div className="text-center text-gray-400 py-8">
            <TrendingUp className="w-12 h-12 mx-auto mb-4 text-gray-500" />
            <p>Prometheus metrics visualization coming soon</p>
            <p className="text-xs mt-2">Use save_prometheus_metric and get_prometheus_metrics commands</p>
          </div>
        </Card>
      )}
    </div>
  );
}
