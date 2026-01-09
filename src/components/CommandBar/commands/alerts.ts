import { Command } from "./types";
import { invoke } from "@tauri-apps/api/core";

export const alertCommands: Command[] = [
  {
    id: "alert-list",
    name: "alert-list",
    description: "List all alerts",
    aliases: ["al", "alerts"],
    category: "Alerts",
    execute: async (args) => {
      const status = args[0] || null; // new, ack, snoozed, resolved
      const limit = args[1] ? parseInt(args[1]) : 50;

      const alerts = await invoke<any[]>("temporal_list_alerts", {
        limit,
        status,
        ruleId: null,
      });

      console.log(`Found ${alerts.length} alerts`);
      alerts.forEach((a) => {
        console.log(`  [${a.status}] ${a.id}: ${JSON.parse(a.payload_json).title || "Alert"}`);
      });
    },
    autocomplete: (args) => {
      if (args.length === 0) {
        return ["new", "ack", "snoozed", "resolved"];
      }
      return [];
    },
  },
  {
    id: "alert-ack",
    name: "alert-ack",
    description: "Acknowledge an alert",
    aliases: ["aa", "ack"],
    category: "Alerts",
    execute: async (args) => {
      const alertId = args[0] ? parseInt(args[0]) : null;
      if (!alertId) {
        throw new Error("Usage: alert-ack <ALERT_ID>");
      }

      await invoke("temporal_acknowledge_alert", { alertId });
      console.log(`Acknowledged alert ${alertId}`);
    },
  },
  {
    id: "alert-snooze",
    name: "alert-snooze",
    description: "Snooze an alert",
    aliases: ["as", "snooze"],
    category: "Alerts",
    execute: async (args) => {
      if (args.length < 2) {
        throw new Error("Usage: alert-snooze <ALERT_ID> <MINUTES>");
      }

      const alertId = parseInt(args[0]);
      const minutes = parseInt(args[1]);
      const until = Math.floor(Date.now() / 1000) + minutes * 60;

      await invoke("temporal_snooze_alert", { alertId, until });
      console.log(`Snoozed alert ${alertId} for ${minutes} minutes`);
    },
  },
  {
    id: "alert-rule-list",
    name: "alert-rule-list",
    description: "List all alert rules",
    aliases: ["arl", "rules"],
    category: "Alerts",
    execute: async () => {
      const rules = await invoke<any[]>("temporal_list_alert_rules");
      
      console.log(`Found ${rules.length} alert rules:`);
      rules.forEach((r) => {
        console.log(`  [${r.enabled ? "✓" : "✗"}] ${r.id}: ${r.name}`);
      });
    },
  },
  {
    id: "alert-rule-enable",
    name: "alert-rule-enable",
    description: "Enable an alert rule",
    aliases: ["are", "enable-rule"],
    category: "Alerts",
    execute: async (args) => {
      const ruleId = args[0] ? parseInt(args[0]) : null;
      if (!ruleId) {
        throw new Error("Usage: alert-rule-enable <RULE_ID>");
      }

      await invoke("temporal_update_alert_rule", {
        id: ruleId,
        enabled: true,
      });
      console.log(`Enabled alert rule ${ruleId}`);
    },
  },
  {
    id: "alert-rule-disable",
    name: "alert-rule-disable",
    description: "Disable an alert rule",
    aliases: ["ard", "disable-rule"],
    category: "Alerts",
    execute: async (args) => {
      const ruleId = args[0] ? parseInt(args[0]) : null;
      if (!ruleId) {
        throw new Error("Usage: alert-rule-disable <RULE_ID>");
      }

      await invoke("temporal_update_alert_rule", {
        id: ruleId,
        enabled: false,
      });
      console.log(`Disabled alert rule ${ruleId}`);
    },
  },
];

