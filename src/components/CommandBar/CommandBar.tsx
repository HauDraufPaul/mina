import { useState, useEffect } from "react";
import { useCommandBar } from "./useCommandBar";
import CommandInput from "./CommandInput";
import CommandSuggestions from "./CommandSuggestions";
import { X, AlertCircle } from "lucide-react";

export default function CommandBar() {
  const {
    isOpen,
    input,
    suggestions,
    selectedSuggestionIndex,
    close,
    setInput,
    executeCommand,
    setSelectedSuggestionIndex,
  } = useCommandBar();

  const [error, setError] = useState<string | null>(null);

  // Reset error when input changes
  useEffect(() => {
    setError(null);
  }, [input]);

  const handleExecute = async () => {
    if (!input.trim()) return;

    setError(null);
    console.log("Executing command:", input);

    try {
      await executeCommand(input);
      setInput("");
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Command failed";
      console.error("Command execution error:", err);
      setError(errorMessage);
      // Don't close on error so user can see the error message
    }
  };

  const handleSuggestionSelect = (suggestion: string) => {
    setInput(suggestion);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedSuggestionIndex(
        Math.min(selectedSuggestionIndex + 1, suggestions.length - 1)
      );
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedSuggestionIndex(Math.max(selectedSuggestionIndex - 1, 0));
    } else if (e.key === "Tab" && suggestions.length > 0) {
      e.preventDefault();
      // Use first suggestion if none selected, otherwise use selected
      const indexToUse = selectedSuggestionIndex >= 0 ? selectedSuggestionIndex : 0;
      const selected = suggestions[indexToUse];
      if (selected) {
        // Parse current input to see if we're completing a command or argument
        const trimmed = input.trim();
        const parts = trimmed.split(/\s+/);
        
        if (parts.length > 1) {
          // We have a command and are typing an argument
          // Complete just the argument part, preserving the command
          const command = parts[0];
          const existingArgs = parts.slice(1, -1); // All args except the last one being typed
          const newInput = existingArgs.length > 0 
            ? `${command} ${existingArgs.join(" ")} ${selected}`
            : `${command} ${selected}`;
          setInput(newInput);
        } else {
          // Just completing the command name
          setInput(selected);
        }
        // Reset selection after completion
        setSelectedSuggestionIndex(-1);
      }
    }
  };

  if (!isOpen) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-20">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/80 backdrop-blur-sm"
        onClick={close}
      />

      {/* Command Bar */}
      <div className="relative w-full max-w-2xl mx-4">
        <div className="glass-card p-4 border border-white/20 shadow-2xl">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400 font-mono">Command Palette</span>
            <button
              onClick={close}
              className="text-gray-400 hover:text-white transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          <CommandInput
            value={input}
            onChange={setInput}
            onExecute={handleExecute}
            onKeyDown={handleKeyDown}
            placeholder="Type a command (e.g., 'go system-monitor')..."
          />

          {error && (
            <div className="mt-2 flex items-center gap-2 text-neon-red text-sm">
              <AlertCircle className="w-4 h-4" />
              <span>{error}</span>
            </div>
          )}

          <CommandSuggestions
            input={input}
            suggestions={suggestions}
            selectedIndex={selectedSuggestionIndex}
            onSelect={handleSuggestionSelect}
          />

          <div className="mt-2 flex items-center justify-between text-xs text-gray-500">
            <div className="flex items-center gap-4">
              <span>↑↓ Navigate</span>
              <span>Tab Complete</span>
              <span>Enter Execute</span>
            </div>
            <span>Esc Close</span>
          </div>
        </div>
      </div>
    </div>
  );
}

