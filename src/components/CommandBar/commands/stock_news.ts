import { Command } from "./types";
import { useStockNewsStore } from "../../../stores/stockNewsStore";

export const stockNewsCommands: Command[] = [
  {
    id: "news",
    name: "news",
    description: "Show stock news panel",
    aliases: ["n"],
    category: "Stock News",
    execute: (_args, context) => {
      context.navigate("/stock-news");
    },
  },
  {
    id: "news-ticker",
    name: "news ticker",
    description: "Show news for a specific ticker",
    aliases: ["nt"],
    category: "Stock News",
    execute: (args, context) => {
      if (args.length === 0) {
        throw new Error("Usage: news ticker <SYMBOL>");
      }
      const ticker = args[0].toUpperCase();
      useStockNewsStore.getState().setSelectedTickers([ticker]);
      context.navigate("/stock-news");
    },
    autocomplete: () => {
      const tickers = useStockNewsStore.getState().tickers;
      return tickers.map((t) => t.symbol);
    },
  },
  {
    id: "news-sp500",
    name: "news sp500",
    description: "Show S&P 500 news",
    aliases: ["nsp"],
    category: "Stock News",
    execute: (_args, context) => {
      useStockNewsStore.getState().setSelectedIndex("SP500");
      context.navigate("/stock-news");
    },
  },
  {
    id: "news-dax",
    name: "news dax",
    description: "Show DAX news",
    aliases: ["ndax"],
    category: "Stock News",
    execute: (_args, context) => {
      useStockNewsStore.getState().setSelectedIndex("DAX");
      context.navigate("/stock-news");
    },
  },
  {
    id: "ticker",
    name: "ticker",
    description: "Filter news by ticker symbol",
    aliases: ["t"],
    category: "Stock News",
    execute: (args, _context) => {
      if (args.length === 0) {
        throw new Error("Usage: ticker <SYMBOL>");
      }
      const ticker = args[0].toUpperCase();
      const { selectedTickers, toggleTicker } = useStockNewsStore.getState();
      if (!selectedTickers.includes(ticker)) {
        toggleTicker(ticker);
      }
    },
    autocomplete: () => {
      const tickers = useStockNewsStore.getState().tickers;
      return tickers.map((t) => t.symbol);
    },
  },
];

