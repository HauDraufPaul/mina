import { useState, useEffect } from "react";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { GitBranch, CheckCircle, Clock, AlertTriangle } from "lucide-react";

interface Migration {
  version: number;
  name: string;
  applied_at: number;
  status: "applied" | "pending" | "failed";
}

export default function MigrationManager() {
  const [migrations, setMigrations] = useState<Migration[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate migration data
    const mockMigrations: Migration[] = [
      {
        version: 1,
        name: "initial_schema",
        applied_at: Date.now() - 86400000,
        status: "applied",
      },
      {
        version: 2,
        name: "add_error_tracking",
        applied_at: Date.now() - 43200000,
        status: "applied",
      },
      {
        version: 3,
        name: "add_config_table",
        applied_at: Date.now() - 21600000,
        status: "applied",
      },
    ];

    setMigrations(mockMigrations);
    setLoading(false);
  }, []);

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "applied":
        return <CheckCircle className="w-5 h-5 text-neon-green" />;
      case "pending":
        return <Clock className="w-5 h-5 text-neon-amber" />;
      case "failed":
        return <AlertTriangle className="w-5 h-5 text-neon-red" />;
      default:
        return null;
    }
  };

  if (loading) {
    return <div className="text-center">Loading migrations...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Migration Manager
        </h1>
        <p className="text-gray-400">Database migrations and schema management</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <Card title="Total Migrations" subtitle="All migrations">
          <div className="text-3xl font-bold text-neon-cyan">{migrations.length}</div>
        </Card>
        <Card title="Applied" subtitle="Successfully applied">
          <div className="text-3xl font-bold text-neon-green">
            {migrations.filter((m) => m.status === "applied").length}
          </div>
        </Card>
        <Card title="Pending" subtitle="Awaiting application">
          <div className="text-3xl font-bold text-neon-amber">
            {migrations.filter((m) => m.status === "pending").length}
          </div>
        </Card>
      </div>

      <Card title="Migration History">
        <div className="space-y-3">
          {migrations.map((migration) => (
            <div
              key={migration.version}
              className="glass-card p-4 flex items-center justify-between"
            >
              <div className="flex items-center gap-4 flex-1">
                {getStatusIcon(migration.status)}
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-mono text-sm text-neon-cyan">
                      v{migration.version}
                    </span>
                    <span className="font-semibold">{migration.name}</span>
                  </div>
                  <div className="text-xs text-gray-400">
                    Applied: {formatTimestamp(migration.applied_at)}
                  </div>
                </div>
              </div>
              <div className="ml-4">
                <span
                  className={`text-xs px-2 py-1 rounded ${
                    migration.status === "applied"
                      ? "bg-neon-green/20 text-neon-green"
                      : migration.status === "pending"
                      ? "bg-neon-amber/20 text-neon-amber"
                      : "bg-neon-red/20 text-neon-red"
                  }`}
                >
                  {migration.status.toUpperCase()}
                </span>
              </div>
            </div>
          ))}
        </div>
      </Card>

      <Card title="Migration Actions">
        <div className="space-y-3">
          <Button variant="primary" className="w-full">
            <GitBranch className="w-4 h-4 mr-2" />
            Run Pending Migrations
          </Button>
          <Button variant="secondary" className="w-full">
            Validate Schema
          </Button>
          <Button variant="secondary" className="w-full">
            Check Integrity
          </Button>
        </div>
      </Card>

      <Card title="Database Information">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <div className="text-sm text-gray-400 mb-1">Schema Version</div>
            <div className="font-mono text-sm">
              {migrations.length > 0
                ? migrations[migrations.length - 1].version
                : "0"}
            </div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">Last Migration</div>
            <div className="font-mono text-sm">
              {migrations.length > 0
                ? formatTimestamp(
                    migrations[migrations.length - 1].applied_at
                  )
                : "Never"}
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}
