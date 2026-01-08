import { useState, useEffect, useRef } from "react";
import { realtimeService, RealtimeEventType } from "../services/realtimeService";
import { useRealtimeStore } from "../stores/realtimeStore";

export interface UseRealtimeDataOptions<T> {
  topic: RealtimeEventType;
  initialData?: T;
  enabled?: boolean;
  transform?: (data: any) => T;
  debounce?: number;
  onUpdate?: (data: T) => void;
  fallbackPolling?: boolean;
  pollingInterval?: number;
  fetchInitialData?: () => Promise<T>;
}

export function useRealtimeData<T = any>(
  topic: RealtimeEventType,
  options: Omit<UseRealtimeDataOptions<T>, "topic"> = {}
) {
  const {
    initialData,
    enabled = true,
    transform,
    debounce = 0,
    onUpdate,
    fallbackPolling = true,
    pollingInterval = 1000,
    fetchInitialData,
  } = options;

  const [data, setData] = useState<T | null>(initialData ?? null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const debounceTimeoutRef = useRef<number | null>(null);
  const pollingIntervalRef = useRef<number | null>(null);
  const globalPaused = useRealtimeStore((state) => state.isPaused);

  // Fetch initial data
  useEffect(() => {
    if (fetchInitialData && enabled) {
      fetchInitialData()
        .then((data) => {
          setData(data);
          setLoading(false);
        })
        .catch((err) => {
          setError(err instanceof Error ? err : new Error("Failed to fetch initial data"));
          setLoading(false);
        });
    } else if (initialData !== undefined) {
      setData(initialData);
      setLoading(false);
    }
  }, [fetchInitialData, initialData, enabled]);

  // Subscribe to realtime updates
  useEffect(() => {
    if (!enabled || globalPaused) {
      setIsConnected(false);
      return;
    }

    const handleUpdate = (updateData: any) => {
      try {
        let transformedData: T;
        if (transform) {
          transformedData = transform(updateData);
        } else {
          transformedData = updateData as T;
        }

        const update = () => {
          setData(transformedData);
          setLastUpdate(new Date());
          setLoading(false);
          setIsConnected(true);
          onUpdate?.(transformedData);
        };

        if (debounce > 0) {
          if (debounceTimeoutRef.current) {
            clearTimeout(debounceTimeoutRef.current);
          }
          debounceTimeoutRef.current = window.setTimeout(update, debounce);
        } else {
          update();
        }
      } catch (err) {
        const error = err instanceof Error ? err : new Error("Data transformation failed");
        setError(error);
        setLoading(false);
      }
    };

    const unsubscribe = realtimeService.subscribe(topic, handleUpdate);
    setIsConnected(true);

    return () => {
      unsubscribe();
      setIsConnected(false);
    };
  }, [topic, enabled, globalPaused, transform, debounce, onUpdate]);

  // Fallback polling if WebSocket is not available
  useEffect(() => {
    if (!enabled || !fallbackPolling || !fetchInitialData) return;

    // Only use polling if we're not getting realtime updates
    const checkConnection = () => {
      if (!isConnected && fetchInitialData) {
        fetchInitialData()
          .then((data) => {
            setData(data);
            setLastUpdate(new Date());
            setLoading(false);
          })
          .catch((err) => {
            // Silent error for polling fallback
            console.debug("Polling fallback error:", err);
          });
      }
    };

    // Poll every interval if not connected
    if (!isConnected) {
      pollingIntervalRef.current = window.setInterval(checkConnection, pollingInterval);
    }

    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
      }
    };
  }, [enabled, fallbackPolling, fetchInitialData, isConnected, pollingInterval]);

  useEffect(() => {
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
      }
    };
  }, []);

  return {
    data,
    loading,
    error,
    lastUpdate,
    isConnected,
  };
}
