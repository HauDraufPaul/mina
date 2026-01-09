import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import Modal from "@/components/ui/Modal";
import { Plus, Trash2, Edit2, TrendingUp, TrendingDown, DollarSign } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface Portfolio {
  id: number;
  name: string;
  created_at: number;
}

interface Holding {
  id: number;
  portfolio_id: number;
  ticker: string;
  quantity: number;
  purchase_price: number;
  purchase_date: number;
}

interface PortfolioValue {
  total_value: number;
  total_cost: number;
  total_gain: number;
  total_gain_percent: number;
  holdings: HoldingValue[];
}

interface HoldingValue {
  ticker: string;
  quantity: number;
  cost_basis: number;
  current_value: number;
  gain: number;
  gain_percent: number;
  current_price: number;
}

export default function PortfolioManager() {
  const [portfolios, setPortfolios] = useState<Portfolio[]>([]);
  const [selectedPortfolio, setSelectedPortfolio] = useState<number | null>(null);
  const [holdings, setHoldings] = useState<Holding[]>([]);
  const [portfolioValue, setPortfolioValue] = useState<PortfolioValue | null>(null);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showAddHoldingModal, setShowAddHoldingModal] = useState(false);
  const [newPortfolioName, setNewPortfolioName] = useState("");
  const [newHoldingTicker, setNewHoldingTicker] = useState("");
  const [newHoldingQuantity, setNewHoldingQuantity] = useState("");
  const [newHoldingPrice, setNewHoldingPrice] = useState("");
  const errorHandler = useErrorHandler();

  useEffect(() => {
    loadPortfolios();
  }, []);

  useEffect(() => {
    if (selectedPortfolio) {
      loadHoldings(selectedPortfolio);
      loadPortfolioValue(selectedPortfolio);
    }
  }, [selectedPortfolio]);

  const loadPortfolios = async () => {
    try {
      setLoading(true);
      const result = await invoke<Portfolio[]>("list_portfolios");
      setPortfolios(result);
      if (result.length > 0 && !selectedPortfolio) {
        setSelectedPortfolio(result[0].id);
      }
    } catch (err) {
      errorHandler.showError("Failed to load portfolios", err);
    } finally {
      setLoading(false);
    }
  };

  const loadHoldings = async (portfolioId: number) => {
    try {
      const result = await invoke<Holding[]>("list_holdings", { portfolioId });
      setHoldings(result);
    } catch (err) {
      errorHandler.showError("Failed to load holdings", err);
    }
  };

  const loadPortfolioValue = async (portfolioId: number) => {
    try {
      const result = await invoke<PortfolioValue>("get_portfolio_value", { portfolioId });
      setPortfolioValue(result);
    } catch (err) {
      errorHandler.showError("Failed to load portfolio value", err);
    }
  };

  const createPortfolio = async () => {
    if (!newPortfolioName.trim()) return;

    try {
      const id = await invoke<number>("create_portfolio", { name: newPortfolioName.trim() });
      setNewPortfolioName("");
      setShowCreateModal(false);
      await loadPortfolios();
      setSelectedPortfolio(id);
    } catch (err) {
      errorHandler.showError("Failed to create portfolio", err);
    }
  };

  const addHolding = async () => {
    if (!selectedPortfolio || !newHoldingTicker.trim() || !newHoldingQuantity || !newHoldingPrice) return;

    try {
      const quantity = parseFloat(newHoldingQuantity);
      const price = parseFloat(newHoldingPrice);
      const purchaseDate = Math.floor(Date.now() / 1000);

      await invoke<number>("add_holding", {
        portfolioId: selectedPortfolio,
        ticker: newHoldingTicker.trim().toUpperCase(),
        quantity,
        purchasePrice: price,
        purchaseDate,
      });

      setNewHoldingTicker("");
      setNewHoldingQuantity("");
      setNewHoldingPrice("");
      setShowAddHoldingModal(false);
      await loadHoldings(selectedPortfolio);
      await loadPortfolioValue(selectedPortfolio);
    } catch (err) {
      errorHandler.showError("Failed to add holding", err);
    }
  };

  const deleteHolding = async (holdingId: number) => {
    if (!selectedPortfolio) return;

    try {
      await invoke("delete_holding", { holdingId });
      await loadHoldings(selectedPortfolio);
      await loadPortfolioValue(selectedPortfolio);
    } catch (err) {
      errorHandler.showError("Failed to delete holding", err);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-200">Portfolio Manager</h2>
          <p className="text-sm text-gray-400">Track and analyze your investment portfolios</p>
        </div>
        <Button variant="primary" onClick={() => setShowCreateModal(true)}>
          <Plus className="w-4 h-4 mr-2" />
          New Portfolio
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card title="Portfolios" subtitle={`${portfolios.length} portfolios`}>
          <div className="space-y-2">
            {portfolios.map((portfolio) => (
              <button
                key={portfolio.id}
                onClick={() => setSelectedPortfolio(portfolio.id)}
                className={`w-full text-left p-3 rounded border transition-colors ${
                  selectedPortfolio === portfolio.id
                    ? "border-neon-cyan bg-neon-cyan/10"
                    : "border-white/10 hover:border-white/20"
                }`}
              >
                <div className="font-semibold">{portfolio.name}</div>
                <div className="text-xs text-gray-400">
                  {new Date(portfolio.created_at * 1000).toLocaleDateString()}
                </div>
              </button>
            ))}
          </div>
        </Card>

        <Card
          title="Holdings"
          subtitle={selectedPortfolio ? `${holdings.length} holdings` : "Select a portfolio"}
        >
          {selectedPortfolio ? (
            <div className="space-y-4">
              <Button
                variant="secondary"
                onClick={() => setShowAddHoldingModal(true)}
                className="w-full"
              >
                <Plus className="w-4 h-4 mr-2" />
                Add Holding
              </Button>

              <div className="space-y-2">
                {holdings.map((holding) => (
                  <div
                    key={holding.id}
                    className="p-3 bg-white/5 border border-white/10 rounded flex items-center justify-between"
                  >
                    <div>
                      <div className="font-mono font-semibold">{holding.ticker}</div>
                      <div className="text-xs text-gray-400">
                        {holding.quantity} @ ${holding.purchase_price.toFixed(2)}
                      </div>
                    </div>
                    <button
                      onClick={() => deleteHolding(holding.id)}
                      className="text-gray-400 hover:text-neon-red transition-colors"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <p>Select a portfolio to view holdings</p>
            </div>
          )}
        </Card>

        <Card
          title="Portfolio Value"
          subtitle={portfolioValue ? "Current valuation" : "No data"}
        >
          {portfolioValue ? (
            <div className="space-y-4">
              <div>
                <div className="text-3xl font-bold text-gray-200">
                  ${portfolioValue.total_value.toLocaleString(undefined, {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 2,
                  })}
                </div>
                <div className="text-sm text-gray-400 mt-1">Total Value</div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-lg font-semibold text-gray-300">
                    ${portfolioValue.total_cost.toLocaleString(undefined, {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 2,
                    })}
                  </div>
                  <div className="text-xs text-gray-400">Cost Basis</div>
                </div>
                <div>
                  <div
                    className={`text-lg font-semibold flex items-center gap-1 ${
                      portfolioValue.total_gain >= 0 ? "text-neon-cyan" : "text-neon-red"
                    }`}
                  >
                    {portfolioValue.total_gain >= 0 ? (
                      <TrendingUp className="w-4 h-4" />
                    ) : (
                      <TrendingDown className="w-4 h-4" />
                    )}
                    ${portfolioValue.total_gain.toLocaleString(undefined, {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 2,
                    })}
                  </div>
                  <div className="text-xs text-gray-400">
                    {portfolioValue.total_gain_percent >= 0 ? "+" : ""}
                    {portfolioValue.total_gain_percent.toFixed(2)}%
                  </div>
                </div>
              </div>

              <div className="pt-4 border-t border-white/10">
                <div className="text-sm font-semibold mb-2">Holdings Breakdown</div>
                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {portfolioValue.holdings.map((holding) => (
                    <div
                      key={holding.ticker}
                      className="flex items-center justify-between text-sm p-2 bg-white/5 rounded"
                    >
                      <div>
                        <div className="font-mono font-semibold">{holding.ticker}</div>
                        <div className="text-xs text-gray-400">
                          {holding.quantity} shares @ ${holding.current_price.toFixed(2)}
                        </div>
                      </div>
                      <div className="text-right">
                        <div className="font-semibold">
                          ${holding.current_value.toLocaleString(undefined, {
                            minimumFractionDigits: 2,
                            maximumFractionDigits: 2,
                          })}
                        </div>
                        <div
                          className={`text-xs ${
                            holding.gain >= 0 ? "text-neon-cyan" : "text-neon-red"
                          }`}
                        >
                          {holding.gain_percent >= 0 ? "+" : ""}
                          {holding.gain_percent.toFixed(2)}%
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center py-8 text-gray-400">
              <p>No portfolio value data</p>
            </div>
          )}
        </Card>
      </div>

      <Modal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        title="Create Portfolio"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Portfolio Name</label>
            <input
              type="text"
              value={newPortfolioName}
              onChange={(e) => setNewPortfolioName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  createPortfolio();
                }
              }}
              placeholder="My Portfolio"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setShowCreateModal(false)}>
              Cancel
            </Button>
            <Button variant="primary" onClick={createPortfolio}>
              Create
            </Button>
          </div>
        </div>
      </Modal>

      <Modal
        isOpen={showAddHoldingModal}
        onClose={() => setShowAddHoldingModal(false)}
        title="Add Holding"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Ticker</label>
            <input
              type="text"
              value={newHoldingTicker}
              onChange={(e) => setNewHoldingTicker(e.target.value.toUpperCase())}
              placeholder="AAPL"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan font-mono"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Quantity</label>
            <input
              type="number"
              value={newHoldingQuantity}
              onChange={(e) => setNewHoldingQuantity(e.target.value)}
              placeholder="10"
              step="0.01"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Purchase Price</label>
            <input
              type="number"
              value={newHoldingPrice}
              onChange={(e) => setNewHoldingPrice(e.target.value)}
              placeholder="150.00"
              step="0.01"
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded text-white placeholder-gray-500 focus:outline-none focus:border-neon-cyan"
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setShowAddHoldingModal(false)}>
              Cancel
            </Button>
            <Button variant="primary" onClick={addHolding}>
              Add
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
