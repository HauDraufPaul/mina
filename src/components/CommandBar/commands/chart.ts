import { Command } from "./types";

export const chartCommands: Command[] = [
  {
    id: "chart-open",
    name: "chart-open",
    description: "Open chart studio for a ticker",
    aliases: ["co", "chart"],
    category: "Chart",
    execute: (args, context) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: chart-open <TICKER> [TIMEFRAME]");
      }

      const timeframe = (args[1] as "1m" | "5m" | "15m" | "1h" | "1d") || "1d";
      context.navigate(`/chart-studio?ticker=${ticker}&timeframe=${timeframe}`);
    },
    autocomplete: (args) => {
      if (args.length === 1) {
        return ["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA"];
      }
      if (args.length === 2) {
        return ["1m", "5m", "15m", "1h", "1d"];
      }
      return [];
    },
  },
  {
    id: "chart-compare",
    name: "chart-compare",
    description: "Open chart with comparison tickers",
    aliases: ["cc", "compare"],
    category: "Chart",
    execute: (args, context) => {
      if (args.length < 2) {
        throw new Error("Usage: chart-compare <TICKER1> <TICKER2> [TICKER3] ...");
      }

      const tickers = args.map((t) => t.toUpperCase()).join(",");
      context.navigate(`/chart-studio?ticker=${tickers}&mode=compare`);
    },
  },
  {
    id: "chart-indicator",
    name: "chart-indicator",
    description: "Add indicator to chart",
    aliases: ["ci", "indicator"],
    category: "Chart",
    execute: (args, context) => {
      if (args.length < 2) {
        throw new Error("Usage: chart-indicator <TICKER> <INDICATOR> [PERIOD]");
      }

      const ticker = args[0].toUpperCase();
      const indicator = args[1].toLowerCase();
      const period = args[2] ? parseInt(args[2]) : 20;

      const validIndicators = ["sma", "ema", "rsi", "macd", "bollinger"];
      if (!validIndicators.includes(indicator)) {
        throw new Error(`Invalid indicator. Valid: ${validIndicators.join(", ")}`);
      }

      context.navigate(`/chart-studio?ticker=${ticker}&indicator=${indicator}&period=${period}`);
    },
    autocomplete: (args) => {
      if (args.length === 1) {
        return ["AAPL", "MSFT", "GOOGL"];
      }
      if (args.length === 2) {
        return ["sma", "ema", "rsi", "macd", "bollinger"];
      }
      return [];
    },
  },
];

