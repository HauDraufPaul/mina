import { getCommand } from "./commands";
import { parseCommand } from "./commands";

interface CommandSuggestionsProps {
  input: string;
  suggestions: string[];
  selectedIndex: number;
  onSelect: (suggestion: string) => void;
}

export default function CommandSuggestions({
  input,
  suggestions,
  selectedIndex,
  onSelect,
}: CommandSuggestionsProps) {
  if (suggestions.length === 0) {
    return null;
  }

  const parsed = parseCommand(input);
  const currentCmd = parsed ? getCommand(parsed.command) : null;

  return (
    <div className="mt-2 border border-white/10 rounded-lg bg-black/80 backdrop-blur-md overflow-hidden">
      <div className="max-h-64 overflow-y-auto">
        {suggestions.map((suggestion, index) => {
          const isSelected = index === selectedIndex;
          // Try to get command info - suggestions might be command names or autocomplete results
          const cmd = getCommand(suggestion);
          // If not a command, it might be an autocomplete result from current command
          const displayText = cmd ? cmd.name : suggestion;
          
          return (
            <button
              key={index}
              onClick={() => onSelect(suggestion)}
              className={`w-full text-left px-4 py-2 flex items-center justify-between transition-colors ${
                isSelected
                  ? "bg-neon-cyan/20 text-neon-cyan"
                  : "text-gray-300 hover:bg-white/5 hover:text-white"
              }`}
            >
              <div className="flex-1">
                <div className="font-mono text-sm">{displayText}</div>
                {cmd && (
                  <div className="text-xs text-gray-400 mt-0.5">{cmd.description}</div>
                )}
              </div>
              {cmd?.category && (
                <span className="text-xs text-gray-500 ml-4">{cmd.category}</span>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}

