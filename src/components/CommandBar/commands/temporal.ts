import { Command } from "./types";

const temporalTabs = [
  "timeline",
  "graph",
  "alerts",
  "watchlists",
  "search",
  "backtests",
  "workbench",
] as const;

export const temporalCommands: Command[] = [
  {
    id: "te",
    name: "te",
    description: "Open Temporal Engine views",
    aliases: ["temporal"],
    category: "Temporal",
    execute: (args, context) => {
      const tab = (args[0] || "timeline").toLowerCase();
      if (!temporalTabs.includes(tab as any)) {
        throw new Error(`Usage: te <${temporalTabs.join("|")}>`);
      }
      context.navigate(`/reality?tab=${tab}`);
    },
    autocomplete: (args) => {
      if (args.length === 0) return [...temporalTabs];
      const q = args[0].toLowerCase();
      return temporalTabs.filter((t) => t.startsWith(q));
    },
  },
];


