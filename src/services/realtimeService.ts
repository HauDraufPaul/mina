import { listen } from "@tauri-apps/api/event";

export interface SystemMetrics {
  cpu: {
    usage: number;
    cores: number;
    frequency: number;
  };
  memory: {
    total: number;
    used: number;
    free: number;
    usage: number;
  };
  disk: {
    total: number;
    used: number;
    free: number;
    usage: number;
  };
  network: {
    rx: number;
    tx: number;
    rxSpeed: number;
    txSpeed: number;
  };
}

export type RealtimeEventType = "system-metrics" | "network-update" | "process-update" | "error" | "config-update";

export interface RealtimeEvent {
  type: RealtimeEventType;
  data: any;
  timestamp: number;
}

class RealtimeService {
  private listeners: Map<RealtimeEventType, Set<(data: any) => void>> = new Map();
  private isListening = false;

  async start() {
    if (this.isListening) return;
    this.isListening = true;

    // Listen for WebSocket messages via Tauri events
    // The backend will emit events that we can listen to
    try {
      await listen<RealtimeEvent>("ws-message", (event) => {
        const { type, data } = event.payload;
        const listeners = this.listeners.get(type as RealtimeEventType);
        if (listeners) {
          listeners.forEach((callback) => callback(data));
        }
      });
    } catch (error) {
      console.error("Failed to start realtime service:", error);
      this.isListening = false;
    }
  }

  subscribe(type: RealtimeEventType, callback: (data: any) => void) {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set());
    }
    this.listeners.get(type)!.add(callback);

    // Start listening if not already started
    if (!this.isListening) {
      this.start().catch((err) => {
        console.error("Failed to start realtime service:", err);
      });
    }

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(type);
      if (listeners) {
        listeners.delete(callback);
      }
    };
  }

  unsubscribe(type: RealtimeEventType, callback: (data: any) => void) {
    const listeners = this.listeners.get(type);
    if (listeners) {
      listeners.delete(callback);
    }
  }
}

export const realtimeService = new RealtimeService();

