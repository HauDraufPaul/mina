import { Command } from "./types";
import { invoke } from "@tauri-apps/api/core";

export const portfolioCommands: Command[] = [
  {
    id: "portfolio-list",
    name: "portfolio-list",
    description: "List all portfolios",
    aliases: ["pl", "portfolios"],
    category: "Portfolio",
    execute: async () => {
      const portfolios = await invoke<any[]>("list_portfolios");
      console.log(`Found ${portfolios.length} portfolios:`);
      portfolios.forEach((p) => {
        console.log(`  ${p.id}: ${p.name}`);
      });
      return portfolios;
    },
  },
  {
    id: "portfolio-create",
    name: "portfolio-create",
    description: "Create a new portfolio",
    aliases: ["pc", "create-portfolio"],
    category: "Portfolio",
    execute: async (args) => {
      const name = args[0];
      if (!name) {
        throw new Error("Usage: portfolio-create <NAME>");
      }

      const id = await invoke<number>("create_portfolio", { name });
      console.log(`Created portfolio: ${name} (ID: ${id})`);
    },
  },
  {
    id: "portfolio-add",
    name: "portfolio-add",
    description: "Add holding to portfolio",
    aliases: ["pa", "add-holding"],
    category: "Portfolio",
    execute: async (args) => {
      if (args.length < 3) {
        throw new Error("Usage: portfolio-add <PORTFOLIO_ID> <TICKER> <QUANTITY> [PRICE]");
      }

      const portfolioId = parseInt(args[0]);
      const ticker = args[1].toUpperCase();
      const quantity = parseFloat(args[2]);
      const price = args[3] ? parseFloat(args[3]) : null;

      const holdingId = await invoke<number>("add_holding", {
        portfolioId,
        ticker,
        quantity,
        price,
      });

      console.log(`Added ${quantity} shares of ${ticker} to portfolio ${portfolioId}`);
    },
  },
  {
    id: "portfolio-remove",
    name: "portfolio-remove",
    description: "Remove holding from portfolio",
    aliases: ["pr", "remove-holding"],
    category: "Portfolio",
    execute: async (args) => {
      if (args.length < 2) {
        throw new Error("Usage: portfolio-remove <PORTFOLIO_ID> <HOLDING_ID>");
      }

      const portfolioId = parseInt(args[0]);
      const holdingId = parseInt(args[1]);

      await invoke("remove_holding", { portfolioId, holdingId });
      console.log(`Removed holding ${holdingId} from portfolio ${portfolioId}`);
    },
  },
  {
    id: "portfolio-value",
    name: "portfolio-value",
    description: "Get portfolio value and performance",
    aliases: ["pv", "value"],
    category: "Portfolio",
    execute: async (args) => {
      const portfolioId = args[0] ? parseInt(args[0]) : null;
      
      let id = portfolioId;
      if (!id) {
        const portfolios = await invoke<any[]>("list_portfolios");
        if (portfolios.length === 0) {
          throw new Error("No portfolios found. Create one first.");
        }
        id = portfolios[0].id;
      }

      const value = await invoke<any>("get_portfolio_value", { portfolioId: id });
      console.log(`Portfolio ${id}:`);
      console.log(`  Total Value: $${value.totalValue?.toFixed(2) || "N/A"}`);
      console.log(`  Total Cost: $${value.totalCost?.toFixed(2) || "N/A"}`);
      console.log(`  Gain/Loss: $${value.gainLoss?.toFixed(2) || "N/A"} (${value.gainLossPercent?.toFixed(2) || "N/A"}%)`);
      
      return value;
    },
  },
  {
    id: "portfolio-analyze",
    name: "portfolio-analyze",
    description: "Analyze portfolio impact of events",
    aliases: ["panalyze", "analyze"],
    category: "Portfolio",
    execute: async (args) => {
      const portfolioId = args[0] ? parseInt(args[0]) : null;
      
      let id = portfolioId;
      if (!id) {
        const portfolios = await invoke<any[]>("list_portfolios");
        if (portfolios.length === 0) {
          throw new Error("No portfolios found. Create one first.");
        }
        id = portfolios[0].id;
      }

      const analysis = await invoke<any>("analyze_portfolio_impact", { portfolioId: id });
      console.log(`Portfolio ${id} Impact Analysis:`);
      console.log(`  Affected Holdings: ${analysis.affectedHoldings?.length || 0}`);
      console.log(`  Total Impact: $${analysis.totalImpact?.toFixed(2) || "N/A"}`);
      
      return analysis;
    },
  },
];

