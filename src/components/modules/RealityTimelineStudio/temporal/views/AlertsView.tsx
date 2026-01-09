import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import { realtimeService } from "@/services/realtimeService";
import { Plus, Bell, Check, Clock, AlertTriangle, Mail, MessageSquare, Webhook, Smartphone } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface AlertRule {
  id: number;
  name: string;
  enabled: boolean;
  watchlist_id?: number | null;
  rule_json: Record<string, unknown>;
  schedule?: string | null;
  escalation_config?: Record<string, unknown>;
  created_at: number;
}

interface AlertEscalation {
  id: number;
  alert_id: number;
  escalation_level: number;
  channel: string;
  sent_at: number | null;
  error: string | null;
  created_at: number;
}

interface Alert {
  id: number;
  rule_id: number;
  fired_at: number;
  event_id?: number | null;
  payload_json: Record<string, unknown>;
  status: string;
  snoozed_until?: number | null;
}

const defaultRuleJson = {
  any: [{ type: "contains_keyword", keyword: "breaking" }],
  all: [],
};

export default function AlertsView() {
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newRuleName, setNewRuleName] = useState("Keyword Alert");
  const [newRuleJsonText, setNewRuleJsonText] = useState(JSON.stringify(defaultRuleJson, null, 2));
  const [escalationConfig, setEscalationConfig] = useState<string>("");
  const [selectedAlert, setSelectedAlert] = useState<Alert | null>(null);
  const [escalationHistory, setEscalationHistory] = useState<AlertEscalation[]>([]);
  const [showEscalateModal, setShowEscalateModal] = useState(false);
  const [escalateLevel, setEscalateLevel] = useState(1);
  const [escalateChannel, setEscalateChannel] = useState("email");
  const errorHandler = useErrorHandler();

  const load = async () => {
    setLoading(true);
    try {
      const [r, a] = await Promise.all([
        invoke<AlertRule[]>("temporal_list_alert_rules"),
        invoke<Alert[]>("temporal_list_alerts", { limit: 200 }),
      ]);
      setRules(r);
      setAlerts(a);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  // Realtime refresh when a new temporal alert is emitted from the backend
  useEffect(() => {
    const unsubscribe = realtimeService.subscribe("temporal-alert", () => {
      load().catch(() => {});
    });
    return () => unsubscribe();
  }, []);

  const createRule = async () => {
    const name = newRuleName.trim();
    if (!name) return;
    let rule_json: Record<string, unknown>;
    try {
      rule_json = JSON.parse(newRuleJsonText);
    } catch (e) {
      errorHandler.showError("Invalid JSON in rule", e);
      return;
    }
    let escalation_config_json = null;
    if (escalationConfig.trim()) {
      try {
        escalation_config_json = JSON.parse(escalationConfig);
      } catch (e) {
        errorHandler.showError("Invalid escalation config JSON", e);
        return;
      }
    }

    await invoke<number>("temporal_create_alert_rule", {
      name,
      enabled: true,
      watchlistId: null,
      ruleJson: rule_json,
      schedule: null,
      escalationConfig: escalation_config_json,
    });
    setShowCreate(false);
    setEscalationConfig("");
    await load();
  };

  const ack = async (id: number) => {
    await invoke("temporal_ack_alert", { alertId: id });
    await load();
  };

  const snooze = async (id: number, seconds: number) => {
    await invoke("temporal_snooze_alert", { alertId: id, snoozeSeconds: seconds });
    await load();
  };

  const resolve = async (id: number) => {
    await invoke("temporal_resolve_alert", { alertId: id });
    await load();
  };

  const labelAlert = async (id: number, label: number) => {
    await invoke("temporal_set_alert_label", { alertId: id, label, note: null });
    await load();
  };

  const loadEscalationHistory = async (alertId: number) => {
    try {
      const history = await invoke<AlertEscalation[]>("get_alert_escalation_history", { alertId });
      setEscalationHistory(history);
    } catch (err) {
      errorHandler.showError("Failed to load escalation history", err);
    }
  };

  const manualEscalate = async (alertId: number, level: number, channel: string) => {
    try {
      await invoke("escalate_alert", { alertId, escalationLevel: level, channel });
      await load();
      if (selectedAlert?.id === alertId) {
        await loadEscalationHistory(alertId);
      }
    } catch (err) {
      errorHandler.showError("Failed to escalate alert", err);
    }
  };

  const getChannelIcon = (channel: string) => {
    switch (channel) {
      case "email":
        return <Mail className="w-3 h-3" />;
      case "sms":
        return <MessageSquare className="w-3 h-3" />;
      case "webhook":
        return <Webhook className="w-3 h-3" />;
      case "push":
        return <Smartphone className="w-3 h-3" />;
      default:
        return <AlertTriangle className="w-3 h-3" />;
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-200">Alerts</h3>
          <p className="text-sm text-gray-400">Rule-based alerting on temporal events (MVP)</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="secondary" onClick={load} disabled={loading}>
            Refresh
          </Button>
          <Button variant="primary" onClick={() => setShowCreate(true)}>
            <Plus className="w-4 h-4 mr-2" />
            New Rule
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card title="Alert Rules" subtitle={`${rules.length} rules`}>
          {rules.length === 0 ? (
            <div className="text-gray-400">No rules yet. Create one.</div>
          ) : (
            <div className="space-y-2">
              {rules.map((r) => (
                <div key={r.id} className="glass-card p-3">
                  <div className="flex items-center justify-between">
                    <div className="font-semibold text-gray-200">
                      {r.name} <span className="text-xs text-gray-500">#{r.id}</span>
                    </div>
                    <span className={`text-xs ${r.enabled ? "text-neon-green" : "text-gray-500"}`}>
                      {r.enabled ? "enabled" : "disabled"}
                    </span>
                  </div>
                  <pre className="text-xs text-gray-400 mt-2 overflow-auto max-h-40">
                    {JSON.stringify(r.rule_json, null, 2)}
                  </pre>
                </div>
              ))}
            </div>
          )}
        </Card>

        <Card title="Alert Feed" subtitle={loading ? "Loading…" : `${alerts.length} alerts`}>
          {loading ? (
            <div className="text-gray-400">Loading…</div>
          ) : alerts.length === 0 ? (
            <div className="text-gray-400">
              No alerts fired yet. Create a rule, then click “Fetch Articles” or “Rebuild Events” in Timeline.\n            </div>
          ) : (
            <div className="space-y-2">
              {alerts.map((a) => (
                <div key={a.id} className="glass-card p-3">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Bell className="w-4 h-4 text-neon-amber" />
                      <div className="text-sm text-gray-200">
                        Alert #{a.id} • rule {a.rule_id}
                        {a.event_id ? ` • event ${a.event_id}` : ""}
                      </div>
                    </div>
                    <span className="text-xs text-gray-500">{new Date(a.fired_at * 1000).toLocaleString()}</span>
                  </div>
                  <div className="text-xs text-gray-500 mt-1">status: {a.status}</div>
                  <div className="flex items-center gap-2 mt-2 flex-wrap">
                    <Button variant="secondary" onClick={() => ack(a.id)}>
                      <Check className="w-4 h-4 mr-2" />
                      Ack
                    </Button>
                    <Button variant="secondary" onClick={() => snooze(a.id, 30 * 60)}>
                      <Clock className="w-4 h-4 mr-2" />
                      Snooze 30m
                    </Button>
                    <Button variant="secondary" onClick={() => resolve(a.id)}>
                      Resolve
                    </Button>
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setSelectedAlert(a);
                        loadEscalationHistory(a.id);
                      }}
                    >
                      <AlertTriangle className="w-4 h-4 mr-2" />
                      Escalations
                    </Button>
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setSelectedAlert(a);
                        setShowEscalateModal(true);
                      }}
                    >
                      <Bell className="w-4 h-4 mr-2" />
                      Escalate
                    </Button>
                    <Button variant="secondary" onClick={() => labelAlert(a.id, 1)}>
                      Helpful
                    </Button>
                    <Button variant="secondary" onClick={() => labelAlert(a.id, -1)}>
                      Not helpful
                    </Button>
                  </div>
                  {selectedAlert?.id === a.id && escalationHistory.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-white/10">
                      <div className="text-xs text-gray-400 mb-2">Escalation History</div>
                      <div className="space-y-1">
                        {escalationHistory.map((esc) => (
                          <div
                            key={esc.id}
                            className="flex items-center justify-between text-xs p-2 bg-white/5 rounded"
                          >
                            <div className="flex items-center gap-2">
                              {getChannelIcon(esc.channel)}
                              <span className="text-gray-300">
                                Level {esc.escalation_level} • {esc.channel}
                              </span>
                            </div>
                            <div className="flex items-center gap-2">
                              {esc.sent_at ? (
                                <span className="text-neon-green">Sent</span>
                              ) : (
                                <span className="text-neon-amber">Pending</span>
                              )}
                              {esc.error && (
                                <span className="text-neon-red text-xs" title={esc.error}>
                                  Error
                                </span>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>

      <Modal isOpen={showCreate} onClose={() => setShowCreate(false)} title="Create Alert Rule (MVP)">
        <div className="space-y-3">
          <div className="text-sm text-gray-400">
            MVP rule format supports `any`/`all` arrays. Conditions will be implemented fully in the next step.\n          </div>
          <input
            value={newRuleName}
            onChange={(e) => setNewRuleName(e.target.value)}
            className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
            placeholder="Rule name"
          />
          <textarea
            value={newRuleJsonText}
            onChange={(e) => setNewRuleJsonText(e.target.value)}
            className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-xs text-white font-mono h-48"
          />
          <div>
            <label className="block text-sm text-gray-400 mb-2">Escalation Config (Optional)</label>
            <textarea
              value={escalationConfig}
              onChange={(e) => setEscalationConfig(e.target.value)}
              placeholder='{"levels": [{"delay_minutes": 5, "channels": ["email"]}, {"delay_minutes": 15, "channels": ["sms", "webhook"]}]}'
              className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-xs text-white font-mono h-32"
            />
            <p className="text-xs text-gray-500 mt-1">
              Configure escalation levels with delay_minutes and channels (email, sms, webhook, push)
            </p>
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setShowCreate(false)}>
              Cancel
            </Button>
            <Button variant="primary" onClick={createRule}>
              Create
            </Button>
          </div>
        </div>
      </Modal>

      <Modal isOpen={showEscalateModal} onClose={() => setShowEscalateModal(false)} title="Manually Escalate Alert">
        {selectedAlert && (
          <div className="space-y-4">
            <div className="text-sm text-gray-400">
              Alert #{selectedAlert.id} • Rule {selectedAlert.rule_id}
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Escalation Level</label>
              <input
                type="number"
                min="1"
                max="10"
                value={escalateLevel}
                onChange={(e) => setEscalateLevel(parseInt(e.target.value) || 1)}
                className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Channel</label>
              <select
                value={escalateChannel}
                onChange={(e) => setEscalateChannel(e.target.value)}
                className="w-full bg-black/50 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
              >
                <option value="email">Email</option>
                <option value="sms">SMS</option>
                <option value="webhook">Webhook</option>
                <option value="push">Push Notification</option>
              </select>
            </div>
            <div className="flex justify-end gap-2">
              <Button variant="secondary" onClick={() => setShowEscalateModal(false)}>
                Cancel
              </Button>
              <Button
                variant="primary"
                onClick={async () => {
                  if (selectedAlert) {
                    await manualEscalate(selectedAlert.id, escalateLevel, escalateChannel);
                    setShowEscalateModal(false);
                  }
                }}
              >
                Escalate
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}


