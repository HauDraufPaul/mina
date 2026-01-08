import { Command } from "./types";

// Map module names to routes
const moduleRoutes: Record<string, string> = {
  "system-monitor": "/system-monitor",
  "network": "/network",
  "ai": "/ai",
  "devops": "/devops",
  "automation": "/automation",
  "packages": "/packages",
  "reality": "/reality",
  "vector-store": "/vector-store",
  "security": "/security",
  "utilities": "/utilities",
  "create": "/create",
  "testing": "/testing",
  "config": "/config",
  "migration": "/migration",
  "websocket": "/websocket",
  "errors": "/errors",
  "rate-limit": "/rate-limit",
  "vector-search": "/vector-search",
  "analytics": "/analytics",
  "stock-news": "/stock-news",
  "news": "/stock-news",
};

export const navigationCommands: Command[] = [
  {
    id: "go",
    name: "go",
    description: "Navigate to a module",
    aliases: ["g", "navigate", "nav"],
    category: "Navigation",
    execute: (args, context) => {
      if (args.length === 0) {
        throw new Error("Usage: go <module>");
      }
      const module = args[0];
      const route = moduleRoutes[module] || `/${module}`;
      console.log("Executing go command, navigating to:", route);
      context.navigate(route);
    },
    autocomplete: (args) => {
      const modules = Object.keys(moduleRoutes);
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
    execute: (_args, context) => {
      console.log("Executing home command, navigating to /");
      context.navigate("/");
    },
  },
];

