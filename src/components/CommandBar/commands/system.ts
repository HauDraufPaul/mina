import { Command } from "./types";
import { useRealtimeStore } from "../../../stores/realtimeStore";
import { useCommandBarStore } from "../../../stores/commandBarStore";

export const systemCommands: Command[] = [
  {
    id: "help",
    name: "help",
    description: "Show help information",
    aliases: ["?"],
    category: "System",
    execute: (_args, _context) => {
      // Help will be shown in the command bar UI
      // This is just a placeholder - could show a modal or list all commands
      console.log("Help command - showing available commands");
    },
    autocomplete: () => [],
  },
  {
    id: "clear",
    name: "clear",
    description: "Clear command history",
    aliases: ["cls"],
    category: "System",
    execute: (_args, _context) => {
      useCommandBarStore.getState().clearHistory();
      console.log("Command history cleared");
    },
  },
  {
    id: "history",
    name: "history",
    description: "Show command history",
    aliases: ["hist"],
    category: "System",
    execute: (_args, _context) => {
      // History will be shown in the command bar UI
      console.log("History command - history is shown in command bar");
    },
  },
  {
    id: "pause",
    name: "pause",
    description: "Pause all real-time updates",
    category: "System",
    execute: (_args, _context) => {
      useRealtimeStore.getState().pause();
      console.log("Real-time updates paused");
    },
  },
  {
    id: "resume",
    name: "resume",
    description: "Resume all real-time updates",
    category: "System",
    execute: (_args, _context) => {
      useRealtimeStore.getState().resume();
      console.log("Real-time updates resumed");
    },
  },
];

