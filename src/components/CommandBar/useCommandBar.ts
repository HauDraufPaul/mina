import { useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useCommandBarStore } from "../../stores/commandBarStore";
import { getCommand, parseCommand, getAllCommands } from "./commands";
import { setNavigateFunction } from "../../utils/navigation";
import Fuse from "fuse.js";

export function useCommandBar() {
  const navigate = useNavigate();
  const {
    isOpen,
    input,
    history,
    suggestions,
    selectedSuggestionIndex,
    open,
    close,
    setInput,
    addToHistory,
    setSuggestions,
    setSelectedSuggestionIndex,
  } = useCommandBarStore();

  // Set navigate function for commands
  useEffect(() => {
    setNavigateFunction(navigate);
  }, [navigate]);

  // Update suggestions based on input
  useEffect(() => {
    if (!isOpen) {
      setSuggestions([]);
      return;
    }

    const trimmed = input.trim();
    if (!trimmed) {
      // Show recent commands or all commands
      const recentCommands = history.slice(0, 5).map((h) => h.command);
      setSuggestions(recentCommands);
      return;
    }

    // Parse command to see if we're in command mode or search mode
    const parsed = parseCommand(trimmed);
    if (!parsed) {
      setSuggestions([]);
      return;
    }

    const { command, args } = parsed;
    const cmd = getCommand(command);

    if (cmd && cmd.autocomplete) {
      // Use command's autocomplete
      const completions = cmd.autocomplete(args);
      setSuggestions(completions);
    } else {
      // Fuzzy search commands
      const allCommands = getAllCommands();
      const fuse = new Fuse(allCommands, {
        keys: ["name", "description", "aliases"],
        threshold: 0.3,
      });
      const results = fuse.search(trimmed);
      setSuggestions(results.slice(0, 10).map((r) => r.item.name));
    }
  }, [input, isOpen, history, setSuggestions]);

  const executeCommand = useCallback(
    async (commandInput: string) => {
      if (!commandInput.trim()) return;

      const parsed = parseCommand(commandInput);
      if (!parsed) return;

      const { command, args } = parsed;
      const cmd = getCommand(command);

      if (!cmd) {
        addToHistory(commandInput, false);
        throw new Error(`Command not found: ${command}`);
      }

      try {
        await cmd.execute(args);
        addToHistory(commandInput, true);
        close();
      } catch (error) {
        addToHistory(commandInput, false);
        throw error;
      }
    },
    [addToHistory, close]
  );

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+K or / to open
      if ((e.ctrlKey && e.key === "k") || (e.key === "/" && !e.ctrlKey && !e.metaKey)) {
        // Don't open if typing in input/textarea
        const target = e.target as HTMLElement;
        if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") {
          return;
        }
        e.preventDefault();
        if (isOpen) {
          close();
        } else {
          open();
        }
      }

      // Escape to close
      if (e.key === "Escape" && isOpen) {
        close();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, open, close]);

  return {
    isOpen,
    input,
    suggestions,
    selectedSuggestionIndex,
    open,
    close,
    setInput,
    executeCommand,
    setSelectedSuggestionIndex,
  };
}

