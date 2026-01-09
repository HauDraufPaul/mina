import { Command } from "./types";
import { invoke } from "@tauri-apps/api/core";

export const settingsCommands: Command[] = [
  {
    id: "api-key-set",
    name: "api-key-set",
    description: "Set API key for a provider",
    aliases: ["ak", "set-key"],
    category: "Settings",
    execute: async (args) => {
      if (args.length < 2) {
        throw new Error("Usage: api-key-set <PROVIDER> <KEY>");
      }

      const provider = args[0].toLowerCase();
      const key = args[1];

      await invoke("set_api_key", { provider, key });
      console.log(`Set API key for ${provider}`);
    },
    autocomplete: (args) => {
      if (args.length === 0) {
        return ["alpha_vantage", "polygon", "trading_economics", "openai", "twilio"];
      }
      return [];
    },
  },
  {
    id: "api-key-get",
    name: "api-key-get",
    description: "Get API key for a provider (shows if set)",
    aliases: ["akg", "get-key"],
    category: "Settings",
    execute: async (args) => {
      const provider = args[0]?.toLowerCase();
      if (!provider) {
        throw new Error("Usage: api-key-get <PROVIDER>");
      }

      try {
        const key = await invoke<string>("get_api_key", { provider });
        console.log(`API key for ${provider}: ${key ? "***" + key.slice(-4) : "Not set"}`);
      } catch (err) {
        console.log(`API key for ${provider}: Not set`);
      }
    },
    autocomplete: (args) => {
      if (args.length === 0) {
        return ["alpha_vantage", "polygon", "trading_economics", "openai", "twilio"];
      }
      return [];
    },
  },
  {
    id: "config",
    name: "config",
    description: "Open configuration/settings",
    aliases: ["settings", "cfg"],
    category: "Settings",
    execute: (_args, context) => {
      context.navigate("/config");
    },
  },
  {
    id: "theme",
    name: "theme",
    description: "Change theme (light/dark)",
    aliases: ["t"],
    category: "Settings",
    execute: async (args) => {
      const theme = args[0]?.toLowerCase();
      if (!theme || !["light", "dark"].includes(theme)) {
        throw new Error("Usage: theme <light|dark>");
      }

      // This would update theme - placeholder for now
      console.log(`Theme set to ${theme}`);
    },
    autocomplete: () => {
      return ["light", "dark"];
    },
  },
];

