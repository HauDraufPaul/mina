import { ComponentType } from "react";
import MarketDataWidget from "./MarketDataWidget";
import PortfolioWidget from "./PortfolioWidget";
import CalendarWidget from "./CalendarWidget";
import AlertsWidget from "./AlertsWidget";
import ChartWidget from "./ChartWidget";

export interface WidgetConfig {
  id: string;
  type: string;
  title: string;
  config: Record<string, unknown>;
}

export interface WidgetProps {
  config: Record<string, unknown>;
  onConfigChange?: (config: Record<string, unknown>) => void;
}

// Registry of all available widgets
const widgetRegistry: Record<string, ComponentType<WidgetProps>> = {
  "market-data": MarketDataWidget,
  "portfolio": PortfolioWidget,
  "calendar": CalendarWidget,
  "alerts": AlertsWidget,
  "chart": ChartWidget,
};

// Widget metadata
export const widgetMetadata: Record<string, { name: string; description: string; defaultConfig: Record<string, unknown> }> = {
  "market-data": {
    name: "Market Data",
    description: "Display real-time market prices for selected tickers",
    defaultConfig: {
      tickers: ["AAPL", "MSFT", "GOOGL"],
      showChange: true,
      showVolume: false,
    },
  },
  "portfolio": {
    name: "Portfolio Summary",
    description: "Show portfolio value and performance",
    defaultConfig: {
      portfolioId: null,
      showHoldings: false,
    },
  },
  "calendar": {
    name: "Economic Calendar",
    description: "Display upcoming economic events",
    defaultConfig: {
      daysAhead: 7,
      country: null,
      showImpact: true,
    },
  },
  "alerts": {
    name: "Recent Alerts",
    description: "Show recent temporal alerts",
    defaultConfig: {
      limit: 10,
      showUnreadOnly: false,
    },
  },
  "chart": {
    name: "Market Chart",
    description: "Mini chart for a ticker",
    defaultConfig: {
      ticker: "AAPL",
      timeframe: "1d",
      showEvents: false,
    },
  },
};

export function getWidgetComponent(type: string): ComponentType<WidgetProps> | null {
  return widgetRegistry[type] || null;
}

export function registerWidget(type: string, component: ComponentType<WidgetProps>) {
  widgetRegistry[type] = component;
}

export function listAvailableWidgets(): string[] {
  return Object.keys(widgetRegistry);
}

export function getWidgetMetadata(type: string) {
  return widgetMetadata[type] || null;
}

