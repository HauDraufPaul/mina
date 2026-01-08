import { useEffect, useRef, useState, useCallback } from "react";

export interface WsMessage {
  type: string;
  data?: any;
}

export interface UseWebSocketOptions {
  topics?: string[];
  autoConnect?: boolean;
  reconnectInterval?: number;
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
    onError,
    onConnect,
    onDisconnect,
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const isConnectingRef = useRef(false);

  const connect = useCallback(async () => {
    if (isConnectingRef.current) return;
    isConnectingRef.current = true;

    try {
      // In Tauri, we'll use invoke to communicate with the backend
      // The backend will handle WebSocket connections internally
      // For now, we'll simulate connection status
      setIsConnected(true);
      setError(null);
      onConnect?.();

      // Subscribe to topics if provided
      if (topics.length > 0) {
        // This will be implemented when we add WebSocket commands
        // await invoke("ws_subscribe", { topics });
      }
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Connection failed");
      setError(error);
      setIsConnected(false);
      onError?.(error);

      // Attempt reconnection
      if (reconnectInterval > 0) {
        reconnectTimeoutRef.current = window.setTimeout(() => {
          connect();
        }, reconnectInterval);
      }
    } finally {
      isConnectingRef.current = false;
    }
  }, [topics, reconnectInterval, onConnect, onError]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    setIsConnected(false);
    onDisconnect?.();
  }, [onDisconnect]);

  const sendMessage = useCallback(async (_message: WsMessage) => {
    if (!isConnected) {
      throw new Error("WebSocket not connected");
    }
    // This will be implemented when we add WebSocket commands
    // await invoke("ws_send_message", { message });
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
    sendMessage,
  };
}

