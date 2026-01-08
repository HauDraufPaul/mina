import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

interface CommandHistoryEntry {
  command: string;
  timestamp: number;
  success: boolean;
}

interface CommandBarState {
  isOpen: boolean;
  input: string;
  history: CommandHistoryEntry[];
  historyIndex: number;
  suggestions: string[];
  selectedSuggestionIndex: number;
  
  open: () => void;
  close: () => void;
  setInput: (input: string) => void;
  addToHistory: (command: string, success: boolean) => void;
  navigateHistory: (direction: "up" | "down") => void;
  setSuggestions: (suggestions: string[]) => void;
  setSelectedSuggestionIndex: (index: number) => void;
  clearHistory: () => void;
}

export const useCommandBarStore = create<CommandBarState>()(
  persist(
    (set, get) => ({
      isOpen: false,
      input: "",
      history: [],
      historyIndex: -1,
      suggestions: [],
      selectedSuggestionIndex: 0,
      
      open: () => set({ isOpen: true, input: "", historyIndex: -1 }),
      close: () => set({ isOpen: false, input: "", suggestions: [], selectedSuggestionIndex: 0 }),
      setInput: (input) => set({ input, historyIndex: -1 }),
      
      addToHistory: (command, success) => {
        const history = get().history;
        const newEntry: CommandHistoryEntry = {
          command,
          timestamp: Date.now(),
          success,
        };
        // Remove duplicates and add to front
        const filtered = history.filter((h) => h.command !== command);
        set({ history: [newEntry, ...filtered].slice(0, 100) });
      },
      
      navigateHistory: (direction) => {
        const { history, historyIndex } = get();
        if (history.length === 0) return;
        
        let newIndex = historyIndex;
        if (direction === "up") {
          newIndex = historyIndex === -1 ? 0 : Math.min(historyIndex + 1, history.length - 1);
        } else {
          newIndex = historyIndex === -1 ? -1 : Math.max(historyIndex - 1, -1);
        }
        
        const entry = newIndex >= 0 ? history[newIndex] : null;
        set({
          historyIndex: newIndex,
          input: entry ? entry.command : "",
        });
      },
      
      setSuggestions: (suggestions) => set({ suggestions, selectedSuggestionIndex: 0 }),
      setSelectedSuggestionIndex: (index) => set({ selectedSuggestionIndex: index }),
      clearHistory: () => set({ history: [] }),
    }),
    {
      name: "command-bar-storage",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ history: state.history }),
    }
  )
);

