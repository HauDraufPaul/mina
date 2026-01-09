import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { realtimeService } from "@/services/realtimeService";

export interface MarketPrice {
  ticker: string;
  price: number;
  change: number;
  change_percent: number;
  volume: number;
  timestamp: number;
}

export interface UseMarketDataOptions {
  tickers?: string[];
  autoSubscribe?: boolean;
  refreshInterval?: number;
}

export function useMarketData(options: UseMarketDataOptions = {}) {
  const {
    tickers = [],
    autoSubscribe = true,
    refreshInterval = 60000, // 1 minute default
  } = options;

  const [prices, setPrices] = useState<Map<string, MarketPrice>>(new Map());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPrices = useCallback(async (tickerList: string[]) => {
    if (tickerList.length === 0) return;

    setLoading(true);
    setError(null);

    try {
      const result = await invoke<MarketPrice[]>("get_market_prices", {
        tickers: tickerList,
      });

      const priceMap = new Map<string, MarketPrice>();
      for (const price of result) {
        priceMap.set(price.ticker, price);
      }

      setPrices((prev) => {
        const updated = new Map(prev);
        for (const [ticker, price] of priceMap) {
          updated.set(ticker, price);
        }
        return updated;
      });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to fetch market data";
      setError(errorMsg);
      console.error("Failed to fetch market prices:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchPrice = useCallback(async (ticker: string) => {
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<MarketPrice | null>("get_market_price", {
        ticker,
      });

      if (result) {
        setPrices((prev) => {
          const updated = new Map(prev);
          updated.set(ticker, result);
          return updated;
        });
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to fetch market data";
      setError(errorMsg);
      console.error("Failed to fetch market price:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  // Subscribe to WebSocket updates
  useEffect(() => {
    if (!autoSubscribe || tickers.length === 0) return;

    // Initial fetch
    fetchPrices(tickers);

    // Subscribe to real-time updates
    const unsubscribeBatch = realtimeService.subscribe("market-data-batch", (data: MarketPrice[]) => {
      setPrices((prev) => {
        const updated = new Map(prev);
        for (const price of data) {
          // Only update if we're subscribed to this ticker
          if (tickers.includes(price.ticker)) {
            updated.set(price.ticker, price);
          }
        }
        return updated;
      });
    });

    const unsubscribeSingle = realtimeService.subscribe("market-data", (data: MarketPrice) => {
      if (tickers.includes(data.ticker)) {
        setPrices((prev) => {
          const updated = new Map(prev);
          updated.set(data.ticker, data);
          return updated;
        });
      }
    });

    // Fallback polling (less frequent when WebSocket is active)
    const interval = setInterval(() => {
      fetchPrices(tickers);
    }, refreshInterval * 10); // Poll 10x less frequently when WebSocket is active

    return () => {
      unsubscribeBatch();
      unsubscribeSingle();
      clearInterval(interval);
    };
  }, [tickers, autoSubscribe, refreshInterval, fetchPrices]);

  const getPrice = useCallback(
    (ticker: string): MarketPrice | undefined => {
      return prices.get(ticker);
    },
    [prices]
  );

  const formatPrice = useCallback((price: MarketPrice | undefined): string => {
    if (!price) return "N/A";
    return `$${price.price.toFixed(2)}`;
  }, []);

  const formatChange = useCallback((price: MarketPrice | undefined): string => {
    if (!price) return "N/A";
    const sign = price.change >= 0 ? "+" : "";
    return `${sign}${price.change.toFixed(2)} (${sign}${price.change_percent.toFixed(2)}%)`;
  }, []);

  return {
    prices,
    loading,
    error,
    fetchPrice,
    fetchPrices,
    getPrice,
    formatPrice,
    formatChange,
  };
}
