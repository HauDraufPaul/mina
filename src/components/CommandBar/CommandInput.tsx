import { useRef, useEffect } from "react";
import { Search } from "lucide-react";

interface CommandInputProps {
  value: string;
  onChange: (value: string) => void;
  onExecute: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  placeholder?: string;
}

export default function CommandInput({
  value,
  onChange,
  onExecute,
  onKeyDown,
  placeholder = "Type a command or search...",
}: CommandInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Focus input when component mounts
    inputRef.current?.focus();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      onExecute();
    } else {
      onKeyDown(e);
    }
  };

  return (
    <div className="relative flex items-center">
      <Search className="absolute left-4 w-4 h-4 text-gray-400" />
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        className="w-full pl-10 pr-4 py-3 bg-black/50 border border-white/10 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan focus:ring-1 focus:ring-neon-cyan font-mono text-sm"
        autoComplete="off"
        spellCheck={false}
      />
    </div>
  );
}

