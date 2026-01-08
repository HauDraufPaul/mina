import { Command } from "./types";
import { useRealtimeStore } from "../../../stores/realtimeStore";

export const systemCommands: Command[] = [
  {
    id: "help",
    name: "help",
    description: "Show help information",
    aliases: ["?", "h"],
    category: "System",
    execute: () => {
      // Help will be shown in the command bar UI
      // This is just a placeholder
    },
    autocomplete: () => [],
  },
  {
    id: "clear",
    name: "clear",
    description: "Clear command history",
    aliases: ["cls"],
    category: "System",
    execute: () => {
      // Clear history will be handled by the command bar store
    },
  },
  {
    id: "history",
    name: "history",
    description: "Show command history",
    aliases: ["hist"],
    category: "System",
    execute: () => {
      // History will be shown in the command bar UI
    },
  },
  {
    id: "pause",
    name: "pause",
    description: "Pause all real-time updates",
    category: "System",
    execute: () => {
      useRealtimeStore.getState().pause();
    },
  },
  {
    id: "resume",
    name: "resume",
    description: "Resume all real-time updates",
    category: "System",
    execute: () => {
      useRealtimeStore.getState().resume();
    },
  },
];

