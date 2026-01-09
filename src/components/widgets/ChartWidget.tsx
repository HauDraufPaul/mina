import { WidgetProps } from "./WidgetRegistry";
import MarketChart from "@/components/charts/MarketChart";

export default function ChartWidget({ config }: WidgetProps) {
  const ticker = (config.ticker as string) || "AAPL";
  const timeframe = (config.timeframe as "1m" | "5m" | "15m" | "1h" | "1d") || "1d";
  const showEvents = config.showEvents === true;
  const height = (config.height as number) || 200;

  return (
    <div className="h-full min-h-[200px]">
      <MarketChart
        ticker={ticker}
        timeframe={timeframe}
        showEvents={showEvents}
        height={height}
      />
    </div>
  );
}

