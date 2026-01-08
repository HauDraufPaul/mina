export interface Command {
  id: string;
  name: string;
  description: string;
  aliases?: string[];
  execute: (args: string[]) => void | Promise<void>;
  autocomplete?: (args: string[]) => string[];
  category?: string;
}

export interface CommandResult {
  success: boolean;
  message?: string;
  data?: any;
}

export interface ParsedCommand {
  command: string;
  args: string[];
  raw: string;
}

