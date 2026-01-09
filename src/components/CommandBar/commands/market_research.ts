import { Command } from "./types";
import { invoke } from "@tauri-apps/api/core";

export const marketResearchCommands: Command[] = [
  {
    id: "temporal-events",
    name: "temporal-events",
    description: "List temporal events for a ticker",
    aliases: ["te", "events"],
    execute: async (args: string[]) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: temporal-events <TICKER> [LAST:<days>]");
      }

      let days = 30;
      if (args[1]?.startsWith("LAST:")) {
        days = parseInt(args[1].split(":")[1]) || 30;
      }

      const now = Math.floor(Date.now() / 1000);
      const fromTs = now - days * 24 * 3600;

      const events = await invoke<any[]>("temporal_list_events", {
        limit: 100,
        fromTs,
        toTs: now,
      });

      const filtered = events.filter(
        (e) =>
          e.title.toUpperCase().includes(ticker) ||
          e.summary.toUpperCase().includes(ticker)
      );

      console.log(`Found ${filtered.length} events for ${ticker}:`, filtered);
    },
  },
  {
    id: "correlate",
    name: "correlate",
    description: "Correlate news sentiment with ticker",
    aliases: ["corr"],
    execute: async (args: string[], context) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: correlate <TICKER> NEWS SENTIMENT");
      }

      const sentiment = await invoke<Record<string, number>>("get_aggregated_sentiment", {
        tickers: [ticker],
      });

      console.log(`Sentiment for ${ticker}:`, sentiment);
    },
  },
  {
    id: "alert-rule-create",
    name: "alert-rule-create",
    description: "Create an alert rule",
    aliases: ["arc", "alert-create"],
    execute: async (args: string[], context) => {
      if (args.length < 2) {
        throw new Error("Usage: alert-rule-create <name> <conditions_json>");
      }

      const name = args[0];
      const conditionsJson = JSON.parse(args.slice(1).join(" "));

      const id = await invoke<number>("temporal_create_alert_rule", {
        name,
        enabled: true,
        watchlistId: null,
        ruleJson: conditionsJson,
        schedule: null,
        escalationConfig: null,
      });

      console.log(`Created alert rule: ${name} (ID: ${id})`);
    },
  },
  {
    id: "portfolio-value",
    name: "portfolio-value",
    description: "Show portfolio value",
    aliases: ["pv", "portfolio"],
    execute: async (args: string[], context) => {
      const portfolioId = args[0] ? parseInt(args[0]) : null;
      if (!portfolioId) {
        // Get first portfolio
        const portfolios = await invoke<any[]>("list_portfolios");
        if (portfolios.length === 0) {
          throw new Error("No portfolios found. Create one first.");
        }
        const id = portfolios[0].id;
        const value = await invoke<any>("get_portfolio_value", { portfolioId: id });
        console.log(`Portfolio value:`, value);
      } else {
        const value = await invoke<any>("get_portfolio_value", { portfolioId });
        console.log(`Portfolio value:`, value);
      }
    },
  },
  {
    id: "chart",
    name: "chart",
    description: "Open chart for a ticker",
    aliases: ["c"],
    execute: async (args: string[], context) => {
      const ticker = args[0]?.toUpperCase();
      if (!ticker) {
        throw new Error("Usage: chart <TICKER> [TIMEFRAME]");
      }

      const timeframe = (args[1] as "1m" | "5m" | "15m" | "1h" | "1d") || "1d";

      // Navigate to chart studio with ticker
      context.navigate(`/chart-studio?ticker=${ticker}&timeframe=${timeframe}`);
    },
  },
  {
    id: "economic-calendar",
    name: "economic-calendar",
    description: "Show economic calendar",
    aliases: ["ec", "calendar"],
    execute: async (args: string[], context) => {
      const country = args[0] || null;
      const now = Math.floor(Date.now() / 1000);
      const fromTs = now - 7 * 24 * 3600;
      const toTs = now + 30 * 24 * 3600;

      const events = await invoke<any[]>("list_economic_events", {
        fromTs,
        toTs,
        country,
        eventType: null,
      });

      console.log(`Found ${events.length} economic events`);
    },
  },
  {
    id: "search",
    name: "search",
    description: "Search temporal events and news",
    aliases: ["s"],
    execute: async (args: string[], context) => {
      const query = args.join(" ");
      if (!query) {
        throw new Error("Usage: search <query>");
      }

      const results = await invoke<any[]>("temporal_search", {
        query,
        limit: 50,
      });

      console.log(`Found ${results.length} results for "${query}"`);
    },
  },
];
