import { useEffect, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStockNewsStore, StockNewsItem, StockTicker } from "../stores/stockNewsStore";
import { realtimeService } from "../services/realtimeService";

export interface UseStockNewsOptions {
  autoFetch?: boolean;
  autoSubscribe?: boolean;
  tickers?: string[];
  limit?: number;
}

export function useStockNews(options: UseStockNewsOptions = {}) {
  const {
    autoFetch = true,
    autoSubscribe = true,
    tickers = [],
    limit = 100,
  } = options;

  const {
    newsItems,
    loading,
    error,
    setNewsItems,
    addNewsItem,
    addNewsItems,
    setTickers,
    setLoading,
    setError,
  } = useStockNewsStore();

  const [isSubscribed, setIsSubscribed] = useState(false);

  // Fetch tickers
  const fetchTickers = useCallback(async (index?: string) => {
    try {
      const result = await invoke<StockTicker[]>("get_stock_tickers", {
        index: index || null,
      });
      setTickers(result);
      return result;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to fetch tickers";
      setError(errorMsg);
      return [];
    }
  }, [setTickers, setError]);

  // Fetch news
  const fetchNews = useCallback(
    async (options?: { tickers?: string[]; limit?: number; since?: number }) => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<StockNewsItem[]>("get_stock_news", {
          tickers: options?.tickers || null,
          limit: options?.limit || limit,
          since: options?.since || null,
        });
        setNewsItems(result);
        return result;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : "Failed to fetch news";
        setError(errorMsg);
        return [];
      } finally {
        setLoading(false);
      }
    },
    [setNewsItems, setLoading, setError, limit]
  );

  // Search news
  const searchNews = useCallback(
    async (query: string, searchTickers?: string[], searchLimit?: number) => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<StockNewsItem[]>("search_stock_news", {
          query,
          tickers: searchTickers || null,
          limit: searchLimit || limit,
        });
        setNewsItems(result);
        return result;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : "Failed to search news";
        setError(errorMsg);
        return [];
      } finally {
        setLoading(false);
      }
    },
    [setNewsItems, setLoading, setError, limit]
  );

  // Get news for specific ticker
  const getNewsForTicker = useCallback(
    async (ticker: string, tickerLimit?: number) => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<StockNewsItem[]>("get_news_for_ticker", {
          ticker,
          limit: tickerLimit || limit,
        });
        return result;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : "Failed to get ticker news";
        setError(errorMsg);
        return [];
      } finally {
        setLoading(false);
      }
    },
    [setLoading, setError, limit]
  );

  // Refresh news (fetch new items)
  const refreshNews = useCallback(async (refreshTickers?: string[]) => {
    try {
      const count = await invoke<number>("refresh_stock_news", {
        tickers: refreshTickers || null,
      });
      return count;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to refresh news";
      setError(errorMsg);
      return 0;
    }
  }, [setError]);

  // Start real-time news stream
  const startNewsStream = useCallback(async () => {
    try {
      await invoke("start_news_stream");
      setIsSubscribed(true);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Failed to start news stream";
      setError(errorMsg);
    }
  }, [setError]);

  // Subscribe to real-time events
  useEffect(() => {
    if (!autoSubscribe) return;

    const unsubscribeNews = realtimeService.subscribe("stock-news", (data: StockNewsItem) => {
      addNewsItem(data);
    });

    const unsubscribeBatch = realtimeService.subscribe(
      "stock-news-batch",
      (data: StockNewsItem[]) => {
        addNewsItems(data);
      }
    );

    return () => {
      unsubscribeNews();
      unsubscribeBatch();
    };
  }, [autoSubscribe, addNewsItem, addNewsItems]);

  // Initial fetch
  useEffect(() => {
    if (autoFetch) {
      fetchTickers();
      fetchNews({ tickers: tickers.length > 0 ? tickers : undefined });
    }
  }, [autoFetch]); // Only run on mount

  // Start news stream on mount
  useEffect(() => {
    if (autoSubscribe && !isSubscribed) {
      startNewsStream();
    }
  }, [autoSubscribe, isSubscribed, startNewsStream]);

  return {
    newsItems,
    loading,
    error,
    fetchTickers,
    fetchNews,
    searchNews,
    getNewsForTicker,
    refreshNews,
    startNewsStream,
    isSubscribed,
  };
}

