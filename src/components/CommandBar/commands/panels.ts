import { Command } from "./types";
import { useGridLayoutStore } from "../../../stores/gridLayoutStore";

export const panelCommands: Command[] = [
  {
    id: "open",
    name: "open",
    description: "Open a module in a new panel",
    aliases: ["o"],
    category: "Panels",
    execute: (args, context) => {
      if (args.length === 0) {
        throw new Error("Usage: open <module>");
      }
      const module = args[0];
      const { addPanel, getCurrentLayout, createLayout, setCurrentLayout } = useGridLayoutStore.getState();
      
      // Ensure we have a layout
      let layout = getCurrentLayout();
      if (!layout) {
        layout = createLayout("Default Layout");
        setCurrentLayout(layout.id);
      }
      
      // Create panel
      const panel = {
        id: `panel-${Date.now()}`,
        type: "module" as const,
        component: module,
        position: { row: 0, col: 0 },
        size: { width: 1, height: 1 },
        config: {},
        minimized: false,
        maximized: false,
        title: module,
      };
      
      addPanel(panel);
      
      // Navigate to grid view
      console.log("Executing open command, navigating to /grid");
      context.navigate("/grid");
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
    id: "grid",
    name: "grid",
    description: "Open grid layout view",
    aliases: ["layout"],
    category: "Panels",
    execute: (_args, context) => {
      console.log("Executing grid command, navigating to /grid");
      context.navigate("/grid");
    },
  },
];

