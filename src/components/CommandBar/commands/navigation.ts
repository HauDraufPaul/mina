import { Command } from "./types";
import { navigate } from "../../../utils/navigation";

export const navigationCommands: Command[] = [
  {
    id: "go",
    name: "go",
    description: "Navigate to a module",
    aliases: ["g", "navigate", "nav"],
    category: "Navigation",
    execute: async (args) => {
      if (args.length === 0) {
        throw new Error("Usage: go <module>");
      }
      const module = args[0];
      await navigate(module);
    },
    autocomplete: (args) => {
      const modules = [
        "system-monitor",
        "network",
        "ai",
        "devops",
        "automation",
        "packages",
        "reality",
        "vector-store",
        "security",
        "utilities",
        "create",
        "testing",
        "config",
        "migration",
        "websocket",
        "errors",
        "rate-limit",
        "vector-search",
        "analytics",
      ];
      if (args.length === 0) {
        return modules;
      }
      const query = args[0].toLowerCase();
      return modules.filter((m) => m.toLowerCase().includes(query));
    },
  },
  {
    id: "home",
    name: "home",
    description: "Navigate to home/dashboard",
    aliases: ["h", "dashboard"],
    category: "Navigation",
    execute: async () => {
      await navigate("/");
    },
  },
];

