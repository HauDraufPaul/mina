import { Command } from "./types";
import { navigationCommands } from "./navigation";
import { systemCommands } from "./system";
import { panelCommands } from "./panels";
import { temporalCommands } from "./temporal";
import { stockNewsCommands } from "./stock_news";

// Registry of all commands
const commandRegistry = new Map<string, Command>();

// Register all commands
[...navigationCommands, ...systemCommands, ...panelCommands, ...temporalCommands, ...stockNewsCommands].forEach((cmd) => {
  commandRegistry.set(cmd.name, cmd);
  if (cmd.aliases) {
    cmd.aliases.forEach((alias) => {
      commandRegistry.set(alias, cmd);
    });
  }
});

export function getCommand(name: string): Command | undefined {
  return commandRegistry.get(name.toLowerCase());
}

export function getAllCommands(): Command[] {
  const seen = new Set<string>();
  const commands: Command[] = [];
  
  commandRegistry.forEach((cmd) => {
    if (!seen.has(cmd.id)) {
      seen.add(cmd.id);
      commands.push(cmd);
    }
  });
  
  return commands.sort((a, b) => a.name.localeCompare(b.name));
}

export function searchCommands(query: string): Command[] {
  const lowerQuery = query.toLowerCase();
  const commands = getAllCommands();
  
  return commands.filter((cmd) => {
    const nameMatch = cmd.name.toLowerCase().includes(lowerQuery);
    const descMatch = cmd.description.toLowerCase().includes(lowerQuery);
    const aliasMatch = cmd.aliases?.some((alias) => 
      alias.toLowerCase().includes(lowerQuery)
    );
    
    return nameMatch || descMatch || aliasMatch;
  });
}

export function parseCommand(input: string): { command: string; args: string[] } | null {
  const trimmed = input.trim();
  if (!trimmed) return null;
  
  const parts = trimmed.split(/\s+/);
  const command = parts[0];
  const args = parts.slice(1);
  
  return { command, args };
}

