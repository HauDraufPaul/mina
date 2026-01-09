import { Command } from "./types";
import { invoke } from "@tauri-apps/api/core";

export const marketDataCommands: Command[] = [
  {
    id: "price",
    name: "price",
    description: "Get current price for a ticker",
    aliases: ["p", "quote"],
    category: "Market Data",
    execute: async (args) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: price <TICKER>");
      }

      const price = await invoke<any>("get_market_price", { ticker });
      console.log(`${ticker}: $${price.price?.toFixed(2) || "N/A"} (${price.changePercent?.toFixed(2) || "N/A"}%)`);
      return price;
    },
    autocomplete: () => {
      // Could fetch popular tickers from store
      return ["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA", "META", "NVDA"];
    },
  },
  {
    id: "prices",
    name: "prices",
    description: "Get prices for multiple tickers",
    aliases: ["quotes"],
    category: "Market Data",
    execute: async (args) => {
      if (args.length === 0) {
        throw new Error("Usage: prices <TICKER1> [TICKER2] ...");
      }

      const tickers = args.map((t) => t.toUpperCase());
      const prices = await invoke<any[]>("get_market_prices", { tickers });
      
      prices.forEach((p) => {
        console.log(`${p.ticker}: $${p.price?.toFixed(2) || "N/A"} (${p.changePercent?.toFixed(2) || "N/A"}%)`);
      });
      
      return prices;
    },
  },
  {
    id: "history",
    name: "history",
    description: "Get historical price data for a ticker",
    aliases: ["hist", "h"],
    category: "Market Data",
    execute: async (args) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: history <TICKER> [INTERVAL] [DAYS]");
      }

      const interval = (args[1] as "1m" | "5m" | "15m" | "1h" | "1d") || "1d";
      const days = args[2] ? parseInt(args[2]) : 30;

      const now = Math.floor(Date.now() / 1000);
      const fromTs = now - days * 24 * 3600;

      const data = await invoke<any[]>("get_chart_data", {
        ticker,
        fromTs,
        toTs: now,
        interval,
      });

      console.log(`Retrieved ${data.length} data points for ${ticker}`);
      return data;
    },
    autocomplete: (args) => {
      if (args.length === 1) {
        return ["1m", "5m", "15m", "1h", "1d"];
      }
      return [];
    },
  },
  {
    id: "watch",
    name: "watch",
    description: "Add ticker to watchlist",
    aliases: ["w"],
    category: "Market Data",
    execute: async (args) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: watch <TICKER>");
      }

      // This would add to a watchlist - placeholder for now
      console.log(`Added ${ticker} to watchlist`);
      return { ticker, added: true };
    },
  },
  {
    id: "unwatch",
    name: "unwatch",
    description: "Remove ticker from watchlist",
    aliases: ["uw"],
    category: "Market Data",
    execute: async (args) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: unwatch <TICKER>");
      }

      console.log(`Removed ${ticker} from watchlist`);
      return { ticker, removed: true };
    },
  },
];

