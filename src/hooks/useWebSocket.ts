import { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface WsMessage {
  type: string;
  data?: unknown;
}

export interface UseWebSocketOptions {
  topics?: string[];
  autoConnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onMessage?: (message: WsMessage) => void;
  onError?: (error: Error) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

export function useWebSocket(options: UseWebSocketOptions = {}) {
  const {
    topics = [],
    autoConnect = true,
    reconnectInterval = 3000,
    maxReconnectAttempts = 10,
    onError,
    onConnect,
    onDisconnect,
    onMessage,
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const isConnectingRef = useRef(false);
  const connectionIdRef = useRef<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  const connect = useCallback(async () => {
    if (isConnectingRef.current) return;
    isConnectingRef.current = true;

    try {
      // Connect to WebSocket server via Tauri command
      const connectionId = await invoke<string>("ws_connect", { topics });
      connectionIdRef.current = connectionId;
      
      // Listen for WebSocket messages via Tauri events
      const unlisten = await listen<WsMessage>("ws-message", (event) => {
        const message = event.payload;
        if (onMessage) {
          onMessage(message);
        }
      });
      unlistenRef.current = unlisten;

      setIsConnected(true);
      setError(null);
      reconnectAttemptsRef.current = 0;
      onConnect?.();

      // Subscribe to topics if provided and connection is established
      if (topics.length > 0 && connectionId) {
        await invoke("ws_subscribe", { connectionId, topics });
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Connection failed");
      setError(error);
      setIsConnected(false);
      onError?.(error);

      // Attempt reconnection with exponential backoff
      if (reconnectInterval > 0 && reconnectAttemptsRef.current < maxReconnectAttempts) {
        reconnectAttemptsRef.current += 1;
        const backoffDelay = reconnectInterval * Math.pow(2, reconnectAttemptsRef.current - 1);
        reconnectTimeoutRef.current = window.setTimeout(() => {
          connect();
        }, backoffDelay);
      } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
        setError(new Error("Max reconnection attempts reached"));
      }
    } finally {
      isConnectingRef.current = false;
    }
  }, [topics, reconnectInterval, maxReconnectAttempts, onConnect, onError, onMessage]);

  const disconnect = useCallback(async () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    // Unsubscribe and disconnect from backend
    if (connectionIdRef.current) {
      try {
        await invoke("ws_disconnect", { connectionId: connectionIdRef.current });
      } catch (err) {
        console.error("Failed to disconnect:", err);
      }
      connectionIdRef.current = null;
    }

    // Unlisten from Tauri events
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }

    setIsConnected(false);
    onDisconnect?.();
  }, [onDisconnect]);

  const subscribe = useCallback(async (newTopics: string[]) => {
    if (!connectionIdRef.current || !isConnected) {
      throw new Error("WebSocket not connected");
    }
    try {
      await invoke("ws_subscribe", { 
        connectionId: connectionIdRef.current, 
        topics: newTopics 
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to subscribe");
      setError(error);
      throw error;
    }
  }, [isConnected]);

  const unsubscribe = useCallback(async (topicsToRemove: string[]) => {
    if (!connectionIdRef.current || !isConnected) {
      throw new Error("WebSocket not connected");
    }
    try {
      await invoke("ws_unsubscribe", { 
        connectionId: connectionIdRef.current, 
        topics: topicsToRemove 
      });
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to unsubscribe");
      setError(error);
      throw error;
    }
  }, [isConnected]);

  const sendMessage = useCallback(async (_message: WsMessage) => {
    if (!isConnected) {
      throw new Error("WebSocket not connected");
    }
    // Messages are sent via Tauri events, not direct WebSocket
    // This can be implemented if needed for client-to-server messages
  }, [isConnected]);

  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    isConnected,
    error,
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    sendMessage,
  };
}

