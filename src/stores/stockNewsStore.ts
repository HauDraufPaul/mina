import { create } from "zustand";

export interface StockNewsItem {
  id: number;
  title: string;
  content: string;
  url: string;
  source: string;
  source_id?: string;
  published_at: number;
  fetched_at: number;
  sentiment?: number;
  relevance_score: number;
  tickers: string[];
  created_at: number;
}

export interface StockTicker {
  id: number;
  symbol: string;
  name: string;
  exchange: string;
  index_name: string;
  created_at: number;
}

interface StockNewsState {
  // News items
  newsItems: StockNewsItem[];
  loading: boolean;
  error: string | null;

  // Tickers
  tickers: StockTicker[];
  selectedTickers: string[];

  // Filters
  selectedIndex: string | null; // "SP500", "DAX", or null for all
  selectedSource: string | null;
  searchQuery: string;
  timeRange: "1h" | "6h" | "24h" | "7d" | "30d" | "all";

  // UI state
  tickerTapeSpeed: number; // pixels per second
  tickerTapePaused: boolean;

  // Actions
  setNewsItems: (items: StockNewsItem[]) => void;
  addNewsItem: (item: StockNewsItem) => void;
  addNewsItems: (items: StockNewsItem[]) => void;
  setTickers: (tickers: StockTicker[]) => void;
  setSelectedTickers: (tickers: string[]) => void;
  toggleTicker: (ticker: string) => void;
  setSelectedIndex: (index: string | null) => void;
  setSelectedSource: (source: string | null) => void;
  setSearchQuery: (query: string) => void;
  setTimeRange: (range: "1h" | "6h" | "24h" | "7d" | "30d" | "all") => void;
  setTickerTapeSpeed: (speed: number) => void;
  setTickerTapePaused: (paused: boolean) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearFilters: () => void;

  // Computed/filtered news
  getFilteredNews: () => StockNewsItem[];
}

export const useStockNewsStore = create<StockNewsState>((set, get) => ({
  // Initial state
  newsItems: [],
  loading: false,
  error: null,
  tickers: [],
  selectedTickers: [],
  selectedIndex: null,
  selectedSource: null,
  searchQuery: "",
  timeRange: "24h",
  tickerTapeSpeed: 50,
  tickerTapePaused: false,

  // Actions
  setNewsItems: (items) => set({ newsItems: items }),
  
  addNewsItem: (item) =>
    set((state) => {
      // Check if item already exists
      const exists = state.newsItems.some((n) => n.id === item.id);
      if (exists) return state;
      
      // Add to beginning of list and sort by published_at
      const updated = [item, ...state.newsItems].sort(
        (a, b) => b.published_at - a.published_at
      );
      
      // Keep only last 1000 items
      return { newsItems: updated.slice(0, 1000) };
    }),

  addNewsItems: (items) =>
    set((state) => {
      // Filter out existing items
      const newItems = items.filter(
        (item) => !state.newsItems.some((n) => n.id === item.id)
      );
      
      if (newItems.length === 0) return state;
      
      // Add and sort
      const updated = [...newItems, ...state.newsItems].sort(
        (a, b) => b.published_at - a.published_at
      );
      
      // Keep only last 1000 items
      return { newsItems: updated.slice(0, 1000) };
    }),

  setTickers: (tickers) => set({ tickers }),
  setSelectedTickers: (tickers) => set({ selectedTickers: tickers }),
  toggleTicker: (ticker) =>
    set((state) => ({
      selectedTickers: state.selectedTickers.includes(ticker)
        ? state.selectedTickers.filter((t) => t !== ticker)
        : [...state.selectedTickers, ticker],
    })),
  setSelectedIndex: (index) => set({ selectedIndex: index }),
  setSelectedSource: (source) => set({ selectedSource: source }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setTimeRange: (range) => set({ timeRange: range }),
  setTickerTapeSpeed: (speed) => set({ tickerTapeSpeed: speed }),
  setTickerTapePaused: (paused) => set({ tickerTapePaused: paused }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  clearFilters: () =>
    set({
      selectedTickers: [],
      selectedIndex: null,
      selectedSource: null,
      searchQuery: "",
      timeRange: "24h",
    }),

  // Computed
  getFilteredNews: () => {
    const state = get();
    let filtered = state.newsItems;

    // Filter by tickers
    if (state.selectedTickers.length > 0) {
      filtered = filtered.filter((item) =>
        item.tickers.some((t) => state.selectedTickers.includes(t))
      );
    }

    // Filter by index
    if (state.selectedIndex) {
      const indexTickers = state.tickers
        .filter((t) => t.index_name === state.selectedIndex)
        .map((t) => t.symbol);
      filtered = filtered.filter((item) =>
        item.tickers.some((t) => indexTickers.includes(t))
      );
    }

    // Filter by source
    if (state.selectedSource) {
      filtered = filtered.filter((item) => item.source === state.selectedSource);
    }

    // Filter by search query
    if (state.searchQuery) {
      const query = state.searchQuery.toLowerCase();
      filtered = filtered.filter(
        (item) =>
          item.title.toLowerCase().includes(query) ||
          item.content.toLowerCase().includes(query) ||
          item.tickers.some((t) => t.toLowerCase().includes(query))
      );
    }

    // Filter by time range
    if (state.timeRange !== "all") {
      const now = Date.now() / 1000;
      const ranges = {
        "1h": 3600,
        "6h": 6 * 3600,
        "24h": 24 * 3600,
        "7d": 7 * 24 * 3600,
        "30d": 30 * 24 * 3600,
      };
      const cutoff = now - ranges[state.timeRange];
      filtered = filtered.filter((item) => item.published_at >= cutoff);
    }

    return filtered;
  },
}));

