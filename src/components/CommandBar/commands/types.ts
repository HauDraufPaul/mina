export interface CommandContext {
  navigate: (path: string) => void;
}

export interface Command {
  id: string;
  name: string;
  description: string;
  aliases?: string[];
  execute: (args: string[], context: CommandContext) => void | Promise<void>;
  autocomplete?: (args: string[]) => string[];
  category?: string;
}

export interface CommandResult {
  success: boolean;
  message?: string;
  data?: unknown;
}

export interface ParsedCommand {
  command: string;
  args: string[];
  raw: string;
}

