import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import { AlertTriangle, CheckCircle } from "lucide-react";

interface ErrorRecord {
  id: number;
  error_type: string;
  message: string;
  stack_trace?: string;
  source?: string;
  severity: string;
  created_at: number;
  resolved_at?: number;
}

export default function ErrorDashboard() {
  const [errors, setErrors] = useState<ErrorRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchErrors = async () => {
      try {
        const data = await invoke<ErrorRecord[]>("get_recent_errors", { limit: 50 });
        setErrors(data);
        setLoading(false);
      } catch (error) {
        console.error("Failed to fetch errors:", error);
        setLoading(false);
      }
    };

    fetchErrors();
    const interval = setInterval(fetchErrors, 5000);

    return () => clearInterval(interval);
  }, []);

  const getSeverityColor = (severity: string) => {
    switch (severity.toLowerCase()) {
      case "critical":
      case "error":
        return "text-neon-red";
      case "warning":
        return "text-neon-amber";
      case "info":
        return "text-neon-cyan";
      default:
        return "text-gray-400";
    }
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  if (loading) {
    return <div className="text-center">Loading errors...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Error Dashboard
        </h1>
        <p className="text-gray-400">Error tracking and analysis</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <Card title="Total Errors" subtitle={`${errors.length} errors`}>
          <div className="text-3xl font-bold text-neon-red">{errors.length}</div>
        </Card>
        <Card title="Unresolved" subtitle="Active errors">
          <div className="text-3xl font-bold text-neon-amber">
            {errors.filter(e => !e.resolved_at).length}
          </div>
        </Card>
        <Card title="Resolved" subtitle="Fixed errors">
          <div className="text-3xl font-bold text-neon-green">
            {errors.filter(e => e.resolved_at).length}
          </div>
        </Card>
      </div>

      <Card title="Recent Errors">
        <div className="space-y-4">
          {errors.length === 0 ? (
            <div className="text-center text-gray-400 py-8">
              <CheckCircle className="w-12 h-12 mx-auto mb-4 text-neon-green" />
              <p>No errors found. System is healthy!</p>
            </div>
          ) : (
            errors.map((error) => (
              <div
                key={error.id}
                className="glass-card p-4 border-l-4 border-neon-red"
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className={`w-5 h-5 ${getSeverityColor(error.severity)}`} />
                    <span className={`font-semibold ${getSeverityColor(error.severity)}`}>
                      {error.error_type}
                    </span>
                    {error.resolved_at && (
                      <span className="text-xs text-neon-green flex items-center gap-1">
                        <CheckCircle className="w-3 h-3" />
                        Resolved
                      </span>
                    )}
                  </div>
                  <span className="text-xs text-gray-400">
                    {formatTimestamp(error.created_at)}
                  </span>
                </div>
                <p className="text-sm text-gray-300 mb-2">{error.message}</p>
                {error.source && (
                  <p className="text-xs text-gray-500">Source: {error.source}</p>
                )}
                {error.stack_trace && (
                  <details className="mt-2">
                    <summary className="text-xs text-gray-400 cursor-pointer hover:text-gray-300">
                      Stack Trace
                    </summary>
                    <pre className="text-xs text-gray-500 mt-2 p-2 bg-black/20 rounded overflow-auto">
                      {error.stack_trace}
                    </pre>
                  </details>
                )}
              </div>
            ))
          )}
        </div>
      </Card>
    </div>
  );
}
