import { useState, useEffect } from "react";
import Modal from "@/components/ui/Modal";
import Button from "@/components/ui/Button";
import { getWidgetMetadata, listAvailableWidgets } from "./WidgetRegistry";

interface WidgetConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  widgetType: string | null;
  currentConfig: Record<string, unknown>;
  onSave: (config: Record<string, unknown>) => void;
}

export default function WidgetConfigModal({
  isOpen,
  onClose,
  widgetType,
  currentConfig,
  onSave,
}: WidgetConfigModalProps) {
  const [config, setConfig] = useState<Record<string, unknown>>(currentConfig);

  useEffect(() => {
    if (widgetType && isOpen) {
      const metadata = getWidgetMetadata(widgetType);
      if (metadata) {
        setConfig({ ...metadata.defaultConfig, ...currentConfig });
      }
    }
  }, [widgetType, isOpen, currentConfig]);

  const metadata = widgetType ? getWidgetMetadata(widgetType) : null;

  const handleSave = () => {
    onSave(config);
    onClose();
  };

  const updateConfig = (key: string, value: unknown) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  };

  if (!widgetType || !metadata) {
    return null;
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={`Configure ${metadata.name}`}>
      <div className="space-y-4">
        <p className="text-sm text-gray-400">{metadata.description}</p>

        {widgetType === "market-data" && (
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Tickers (comma-separated)</label>
              <input
                type="text"
                value={(config.tickers as string[])?.join(", ") || ""}
                onChange={(e) => {
                  const tickers = e.target.value
                    .split(",")
                    .map((t) => t.trim().toUpperCase())
                    .filter((t) => t.length > 0);
                  updateConfig("tickers", tickers);
                }}
                placeholder="AAPL, MSFT, GOOGL"
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan font-mono"
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.showChange !== false}
                onChange={(e) => updateConfig("showChange", e.target.checked)}
                className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
              />
              <label className="text-sm text-gray-300">Show price change</label>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.showVolume === true}
                onChange={(e) => updateConfig("showVolume", e.target.checked)}
                className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
              />
              <label className="text-sm text-gray-300">Show volume</label>
            </div>
          </div>
        )}

        {widgetType === "portfolio" && (
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Portfolio ID (leave empty for first portfolio)</label>
              <input
                type="number"
                value={(config.portfolioId as number) || ""}
                onChange={(e) => {
                  const value = e.target.value ? parseInt(e.target.value) : null;
                  updateConfig("portfolioId", value);
                }}
                placeholder="Auto-select first"
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.showHoldings === true}
                onChange={(e) => updateConfig("showHoldings", e.target.checked)}
                className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
              />
              <label className="text-sm text-gray-300">Show top holdings</label>
            </div>
          </div>
        )}

        {widgetType === "calendar" && (
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Days Ahead</label>
              <input
                type="number"
                value={(config.daysAhead as number) || 7}
                onChange={(e) => updateConfig("daysAhead", parseInt(e.target.value) || 7)}
                min={1}
                max={30}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Country (optional)</label>
              <input
                type="text"
                value={(config.country as string) || ""}
                onChange={(e) => updateConfig("country", e.target.value || null)}
                placeholder="US, EU, etc."
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.showImpact !== false}
                onChange={(e) => updateConfig("showImpact", e.target.checked)}
                className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
              />
              <label className="text-sm text-gray-300">Show impact scores</label>
            </div>
          </div>
        )}

        {widgetType === "alerts" && (
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Alert Limit</label>
              <input
                type="number"
                value={(config.limit as number) || 10}
                onChange={(e) => updateConfig("limit", parseInt(e.target.value) || 10)}
                min={1}
                max={50}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
              />
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.showUnreadOnly === true}
                onChange={(e) => updateConfig("showUnreadOnly", e.target.checked)}
                className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
              />
              <label className="text-sm text-gray-300">Show unread only</label>
            </div>
          </div>
        )}

        {widgetType === "chart" && (
          <div className="space-y-3">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Ticker</label>
              <input
                type="text"
                value={(config.ticker as string) || "AAPL"}
                onChange={(e) => updateConfig("ticker", e.target.value.toUpperCase())}
                placeholder="AAPL"
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan font-mono"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-2">Timeframe</label>
              <select
                value={(config.timeframe as string) || "1d"}
                onChange={(e) => updateConfig("timeframe", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white focus:outline-none focus:border-neon-cyan"
              >
                <option value="1m">1 Minute</option>
                <option value="5m">5 Minutes</option>
                <option value="15m">15 Minutes</option>
                <option value="1h">1 Hour</option>
                <option value="1d">1 Day</option>
              </select>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.showEvents === true}
                onChange={(e) => updateConfig("showEvents", e.target.checked)}
                className="w-4 h-4 rounded bg-white/5 border-white/10 text-neon-cyan focus:ring-neon-cyan"
              />
              <label className="text-sm text-gray-300">Show temporal events</label>
            </div>
          </div>
        )}

        <div className="flex justify-end gap-2 pt-4 border-t border-white/10">
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSave}>
            Save
          </Button>
        </div>
      </div>
    </Modal>
  );
}

