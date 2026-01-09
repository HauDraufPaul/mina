import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import { TrendingUp, TrendingDown, AlertTriangle } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface ImpactAnalysis {
  portfolio_id: number;
  event_id: number;
  total_impact: number;
  impact_percent: number;
  affected_holdings: HoldingImpact[];
}

interface HoldingImpact {
  ticker: string;
  quantity: number;
  price_change: number;
  price_change_percent: number;
  impact: number;
  impact_percent: number;
}

interface ImpactAnalysisProps {
  portfolioId: number;
  eventId: number;
  priceChanges: Record<string, number>; // ticker -> price change percent
}

export default function ImpactAnalysis({ portfolioId, eventId, priceChanges }: ImpactAnalysisProps) {
  const [analysis, setAnalysis] = useState<ImpactAnalysis | null>(null);
  const [loading, setLoading] = useState(true);
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadAnalysis();
  }, [portfolioId, eventId, priceChanges]);

  const loadAnalysis = async () => {
    try {
      setLoading(true);
      const result = await invoke<ImpactAnalysis>("get_portfolio_impact", {
        portfolioId,
        eventId,
        priceChanges,
      });
      setAnalysis(result);
    } catch (err) {
      errorHandler.showError("Failed to load impact analysis", err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <Card title="Impact Analysis" subtitle="Calculating...">
        <div className="text-center py-8 text-gray-400">Loading...</div>
      </Card>
    );
  }

  if (!analysis) {
    return (
      <Card title="Impact Analysis" subtitle="No data">
        <div className="text-center py-8 text-gray-400">No impact data available</div>
      </Card>
    );
  }

  return (
    <Card title="Impact Analysis" subtitle={`Event #${eventId}`}>
      <div className="space-y-4">
        <div>
          <div className="text-2xl font-bold text-gray-200">
            ${analysis.total_impact.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </div>
          <div className="text-sm text-gray-400 mt-1">Total Impact</div>
          <div
            className={`text-lg font-semibold mt-2 flex items-center gap-1 ${
              analysis.total_impact >= 0 ? "text-neon-cyan" : "text-neon-red"
            }`}
          >
            {analysis.total_impact >= 0 ? (
              <TrendingUp className="w-4 h-4" />
            ) : (
              <TrendingDown className="w-4 h-4" />
            )}
            {analysis.impact_percent >= 0 ? "+" : ""}
            {analysis.impact_percent.toFixed(2)}%
          </div>
        </div>

        {analysis.affected_holdings.length > 0 && (
          <div>
            <div className="text-sm font-semibold mb-2">Affected Holdings</div>
            <div className="space-y-2">
              {analysis.affected_holdings.map((holding) => (
                <div
                  key={holding.ticker}
                  className="p-3 bg-white/5 border border-white/10 rounded"
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="font-mono font-semibold">{holding.ticker}</div>
                    <div
                      className={`text-sm font-semibold ${
                        holding.impact >= 0 ? "text-neon-cyan" : "text-neon-red"
                      }`}
                    >
                      {holding.impact >= 0 ? "+" : ""}$
                      {holding.impact.toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 2,
                      })}
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-xs text-gray-400">
                    <div>
                      <span>Price Change: </span>
                      <span
                        className={
                          holding.price_change_percent >= 0 ? "text-neon-cyan" : "text-neon-red"
                        }
                      >
                        {holding.price_change_percent >= 0 ? "+" : ""}
                        {holding.price_change_percent.toFixed(2)}%
                      </span>
                    </div>
                    <div>
                      <span>Impact: </span>
                      <span
                        className={
                          holding.impact_percent >= 0 ? "text-neon-cyan" : "text-neon-red"
                        }
                      >
                        {holding.impact_percent >= 0 ? "+" : ""}
                        {holding.impact_percent.toFixed(2)}%
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {analysis.affected_holdings.length === 0 && (
          <div className="text-center py-4 text-gray-400">
            <AlertTriangle className="w-8 h-8 mx-auto mb-2 text-gray-500" />
            <p>No holdings affected by this event</p>
          </div>
        )}
      </div>
    </Card>
  );
}
