import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WidgetProps } from "./WidgetRegistry";
import { TrendingUp, TrendingDown, DollarSign } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface PortfolioValue {
  total_value: number;
  total_cost: number;
  total_gain: number;
  total_gain_percent: number;
  holdings: Array<{
    ticker: string;
    quantity: number;
    cost_basis: number;
    current_value: number;
    gain: number;
    gain_percent: number;
    current_price: number;
  }>;
}

export default function PortfolioWidget({ config }: WidgetProps) {
  const portfolioId = config.portfolioId as number | null;
  const showHoldings = config.showHoldings === true;
  const [portfolioValue, setPortfolioValue] = useState<PortfolioValue | null>(null);
  const [loading, setLoading] = useState(true);
  const errorHandler = useErrorHandler();

  useEffect(() => {
    if (portfolioId) {
      loadPortfolioValue();
    } else {
      // Try to get first portfolio
      loadFirstPortfolio();
    }
  }, [portfolioId]);

  const loadFirstPortfolio = async () => {
    try {
      const portfolios = await invoke<any[]>("list_portfolios");
      if (portfolios.length > 0) {
        await loadPortfolioValueForId(portfolios[0].id);
      } else {
        setLoading(false);
      }
    } catch (err) {
      errorHandler.showError("Failed to load portfolios", err);
      setLoading(false);
    }
  };

  const loadPortfolioValue = async () => {
    if (portfolioId) {
      await loadPortfolioValueForId(portfolioId);
    }
  };

  const loadPortfolioValueForId = async (id: number) => {
    try {
      setLoading(true);
      const value = await invoke<PortfolioValue>("get_portfolio_value", { portfolioId: id });
      setPortfolioValue(value);
    } catch (err) {
      errorHandler.showError("Failed to load portfolio value", err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">Loading portfolio...</div>
    );
  }

  if (!portfolioValue) {
    return (
      <div className="p-4 text-center text-gray-400 text-sm">
        <p>No portfolio selected</p>
        <p className="text-xs mt-1">Configure widget to select a portfolio</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div>
        <div className="flex items-center gap-2 mb-1">
          <DollarSign className="w-4 h-4 text-neon-cyan" />
          <div className="text-lg font-bold text-gray-200">
            ${portfolioValue.total_value.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
        </div>
        <div className="text-xs text-gray-400">Total Value</div>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div>
          <div className="text-xs text-gray-400">Gain/Loss</div>
          <div
            className={`text-sm font-semibold flex items-center gap-1 ${
              portfolioValue.total_gain >= 0 ? "text-neon-cyan" : "text-neon-red"
            }`}
          >
            {portfolioValue.total_gain >= 0 ? (
              <TrendingUp className="w-3 h-3" />
            ) : (
              <TrendingDown className="w-3 h-3" />
            )}
            <span>
              {portfolioValue.total_gain >= 0 ? "+" : ""}
              {portfolioValue.total_gain_percent.toFixed(2)}%
            </span>
          </div>
        </div>
        <div>
          <div className="text-xs text-gray-400">Cost Basis</div>
          <div className="text-sm font-semibold text-gray-300">
            ${portfolioValue.total_cost.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
        </div>
      </div>

      {showHoldings && portfolioValue.holdings.length > 0 && (
        <div className="pt-2 border-t border-white/10">
          <div className="text-xs text-gray-400 mb-2">Top Holdings</div>
          <div className="space-y-1">
            {portfolioValue.holdings.slice(0, 3).map((holding) => (
              <div
                key={holding.ticker}
                className="flex items-center justify-between text-xs p-1 bg-white/5 rounded"
              >
                <span className="font-mono">{holding.ticker}</span>
                <span
                  className={
                    holding.gain >= 0 ? "text-neon-cyan" : "text-neon-red"
                  }
                >
                  {holding.gain_percent >= 0 ? "+" : ""}
                  {holding.gain_percent.toFixed(1)}%
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

