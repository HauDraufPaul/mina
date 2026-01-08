import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";

interface BacktestReport {
  from_ts: number;
  to_ts: number;
  total_alerts: number;
  acked_alerts: number;
  snoozed_alerts: number;
  resolved_alerts: number;
  helpful_alerts: number;
  unhelpful_alerts: number;
  by_rule: Record<string, number>;
  by_rule_helpful: Record<string, number>;
  by_rule_unhelpful: Record<string, number>;
}

export default function BacktestsView() {
  const [fromDays, setFromDays] = useState(7);
  const [report, setReport] = useState<BacktestReport | null>(null);
  const [loading, setLoading] = useState(false);

  const run = async () => {
    setLoading(true);
    try {
      const now = Math.floor(Date.now() / 1000);
      const from = now - fromDays * 24 * 3600;
      const res = await invoke<BacktestReport>("temporal_run_backtest_mvp", { fromTs: from, toTs: now });
      setReport(res);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <Card title="Backtests (OSINT-only MVP)" subtitle="Evaluate alert activity and engagement over time">
        <div className="flex items-center gap-2">
          <input
            type="number"
            min={1}
            max={365}
            value={fromDays}
            onChange={(e) => setFromDays(Number(e.target.value))}
            className="w-28 bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
          />
          <span className="text-sm text-gray-400">days back</span>
          <Button variant="primary" onClick={run} disabled={loading}>
            {loading ? "Running…" : "Run Backtest"}
          </Button>
        </div>
      </Card>

      <Card title="Report" subtitle={report ? `From ${new Date(report.from_ts * 1000).toLocaleString()}` : "No report yet"}>
        {!report ? (
          <div className="text-gray-400">Run a backtest to see results.</div>
        ) : (
          <div className="space-y-3">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="glass-card p-3">
                <div className="text-xs text-gray-500">Total Alerts</div>
                <div className="text-2xl font-bold text-neon-cyan">{report.total_alerts}</div>
              </div>
              <div className="glass-card p-3">
                <div className="text-xs text-gray-500">Acked</div>
                <div className="text-2xl font-bold text-neon-green">{report.acked_alerts}</div>
              </div>
              <div className="glass-card p-3">
                <div className="text-xs text-gray-500">Snoozed</div>
                <div className="text-2xl font-bold text-neon-amber">{report.snoozed_alerts}</div>
              </div>
              <div className="glass-card p-3">
                <div className="text-xs text-gray-500">Resolved</div>
                <div className="text-2xl font-bold text-gray-200">{report.resolved_alerts}</div>
              </div>
              <div className="glass-card p-3">
                <div className="text-xs text-gray-500">Helpful</div>
                <div className="text-2xl font-bold text-neon-green">{report.helpful_alerts}</div>
              </div>
              <div className="glass-card p-3">
                <div className="text-xs text-gray-500">Not helpful</div>
                <div className="text-2xl font-bold text-neon-red">{report.unhelpful_alerts}</div>
              </div>
            </div>

            <Card title="Alerts by Rule">
              {Object.keys(report.by_rule).length === 0 ? (
                <div className="text-gray-400">No alerts in the selected period.</div>
              ) : (
                <div className="space-y-2">
                  {Object.entries(report.by_rule).map(([ruleId, count]) => (
                    <div key={ruleId} className="glass-card p-3 flex items-center justify-between">
                      <div className="text-sm text-gray-200">
                        Rule {ruleId}
                        <span className="text-xs text-gray-500 ml-2">
                          helpful {report.by_rule_helpful[ruleId] ?? 0} / unhelpful {report.by_rule_unhelpful[ruleId] ?? 0}
                        </span>
                      </div>
                      <span className="text-sm text-neon-cyan font-semibold">{count}</span>
                    </div>
                  ))}
                </div>
              )}
            </Card>
          </div>
        )}
      </Card>
    </div>
  );
}


