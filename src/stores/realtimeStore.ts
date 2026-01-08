import { create } from "zustand";

interface RealtimeState {
  isPaused: boolean;
  pause: () => void;
  resume: () => void;
  toggle: () => void;
}

export const useRealtimeStore = create<RealtimeState>((set) => ({
  isPaused: false,
  pause: () => set({ isPaused: true }),
  resume: () => set({ isPaused: false }),
  toggle: () => set((state) => ({ isPaused: !state.isPaused })),
}));

