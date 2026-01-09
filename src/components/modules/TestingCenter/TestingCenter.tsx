import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { TestTube, Play, CheckCircle, XCircle, Clock } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface TestResult {
  id: number;
  suite_id: number;
  name: string;
  status: string;
  duration?: number;
  error?: string;
  executed_at: number;
}

interface TestSuite {
  id: number;
  name: string;
  test_type: string;
  created_at: number;
}

interface TestSuiteStats {
  total: number;
  passed: number;
  failed: number;
  duration: number;
}

export default function TestingCenter() {
  const errorHandler = useErrorHandler();
  const [testSuites, setTestSuites] = useState<TestSuite[]>([]);
  const [suiteResults, setSuiteResults] = useState<Record<number, TestResult[]>>({});
  const [suiteStats, setSuiteStats] = useState<Record<number, TestSuiteStats>>({});
  const [selectedSuite, setSelectedSuite] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSuites();
  }, []);

  useEffect(() => {
    if (selectedSuite !== null) {
      loadSuiteData(selectedSuite);
    }
  }, [selectedSuite]);

  const loadSuites = async () => {
    try {
      const suites = await invoke<TestSuite[]>("list_test_suites");
      console.log("Test suites loaded:", suites);
      setTestSuites(suites || []);
      if (suites && suites.length > 0 && selectedSuite === null) {
        setSelectedSuite(suites[0].id);
      }
      setLoading(false);
    } catch (error) {
      errorHandler.showError("Failed to load test suites", error);
      setTestSuites([]);
      setLoading(false);
    }
  };

  const loadSuiteData = async (suiteId: number) => {
    try {
      const [results, stats] = await Promise.all([
        invoke<TestResult[]>("get_suite_results", { suiteId }),
        invoke<TestSuiteStats>("get_suite_stats", { suiteId }),
      ]);
      console.log(`Suite ${suiteId} data loaded:`, { results, stats });
      setSuiteResults({ ...suiteResults, [suiteId]: results || [] });
      setSuiteStats({ ...suiteStats, [suiteId]: stats || { total: 0, passed: 0, failed: 0, duration: 0 } });
    } catch (error) {
      errorHandler.showError("Failed to load suite data", error);
      setSuiteResults({ ...suiteResults, [suiteId]: [] });
      setSuiteStats({ ...suiteStats, [suiteId]: { total: 0, passed: 0, failed: 0, duration: 0 } });
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case "passed":
        return <CheckCircle className="w-4 h-4 text-neon-green" />;
      case "failed":
        return <XCircle className="w-4 h-4 text-neon-red" />;
      case "running":
        return <Clock className="w-4 h-4 text-neon-amber animate-spin" />;
      default:
        return <Clock className="w-4 h-4 text-gray-400" />;
    }
  };


  const totalTests = Object.values(suiteStats).reduce((sum, stats) => sum + stats.total, 0);
  const totalPassed = Object.values(suiteStats).reduce((sum, stats) => sum + stats.passed, 0);
  const totalFailed = Object.values(suiteStats).reduce((sum, stats) => sum + stats.failed, 0);
  const totalDuration = Object.values(suiteStats).reduce((sum, stats) => sum + stats.duration, 0);
  const coverage = totalTests > 0 ? (totalPassed / totalTests) * 100 : 0;

  if (loading) {
    return <div className="text-center">Loading test data...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Testing Center
          </h1>
          <p className="text-gray-400">Test suite and coverage analytics</p>
        </div>
        <div className="flex gap-2">
          <Button onClick={loadSuites} variant="primary">
            <Play className="w-4 h-4 mr-2" />
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
        <Card title="Total Tests" subtitle="All test suites">
          <div className="text-3xl font-bold text-neon-cyan">{totalTests}</div>
        </Card>
        <Card title="Passed" subtitle="Successful tests">
          <div className="text-3xl font-bold text-neon-green">{totalPassed}</div>
        </Card>
        <Card title="Failed" subtitle="Failed tests">
          <div className="text-3xl font-bold text-neon-red">{totalFailed}</div>
        </Card>
        <Card title="Coverage" subtitle="Test coverage">
          <div className="text-3xl font-bold text-neon-amber">{coverage.toFixed(1)}%</div>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {testSuites.map((suite) => {
          const results = suiteResults[suite.id] || [];
          const stats = suiteStats[suite.id] || { total: 0, passed: 0, failed: 0, duration: 0 };
          const isSelected = selectedSuite === suite.id;

          return (
            <Card
              key={suite.id}
              title={suite.name}
              className={isSelected ? "border-2 border-neon-cyan" : ""}
            >
              <div className="space-y-4">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center gap-2">
                    <TestTube className="w-5 h-5 text-neon-cyan" />
                    <span className="font-semibold">{suite.name}</span>
                    <span className="text-xs text-gray-400">({suite.test_type})</span>
                  </div>
                  <div className="text-sm text-gray-400">
                    {stats.duration.toFixed(2)}s
                  </div>
                </div>

                <div className="grid grid-cols-3 gap-2 mb-4">
                  <div className="text-center">
                    <div className="text-lg font-bold text-neon-cyan">{stats.total}</div>
                    <div className="text-xs text-gray-400">Total</div>
                  </div>
                  <div className="text-center">
                    <div className="text-lg font-bold text-neon-green">{stats.passed}</div>
                    <div className="text-xs text-gray-400">Passed</div>
                  </div>
                  <div className="text-center">
                    <div className="text-lg font-bold text-neon-red">{stats.failed}</div>
                    <div className="text-xs text-gray-400">Failed</div>
                  </div>
                </div>

                {stats.total > 0 && (
                  <div className="w-full bg-gray-800 rounded-full h-2 mb-4">
                    <div
                      className="bg-neon-green h-2 rounded-full transition-all"
                      style={{ width: `${(stats.passed / stats.total) * 100}%` }}
                    />
                  </div>
                )}

                <div className="space-y-2 max-h-64 overflow-y-auto">
                  {results.length === 0 ? (
                    <div className="text-center text-gray-400 py-4 text-sm">
                      No test results yet
                    </div>
                  ) : (
                    results.map((test) => (
                      <div
                        key={test.id}
                        className="glass-card p-2 flex items-center justify-between text-sm"
                      >
                        <div className="flex items-center gap-2 flex-1 min-w-0">
                          {getStatusIcon(test.status)}
                          <span className="truncate font-mono text-xs">{test.name}</span>
                        </div>
                        <div className="flex items-center gap-2 ml-2">
                          {test.duration && (
                            <span className="text-xs text-gray-400">
                              {test.duration.toFixed(2)}s
                            </span>
                          )}
                        </div>
                      </div>
                    ))
                  )}
                </div>

                <Button
                  onClick={() => setSelectedSuite(suite.id)}
                  variant="secondary"
                  className="w-full"
                >
                  <Play className="w-4 h-4 mr-2" />
                  View Details
                </Button>
              </div>
            </Card>
          );
        })}
      </div>

      {testSuites.length === 0 && (
        <Card title="No Test Suites">
          <div className="text-center py-8 text-gray-400">
            <TestTube className="w-12 h-12 mx-auto mb-4 text-gray-500" />
            <p>No test suites found. Create test suites to get started.</p>
          </div>
        </Card>
      )}

      <Card title="Test Analytics">
        <div className="grid grid-cols-2 gap-4">
          <div className="flex items-center justify-between p-3 glass-card">
            <span className="text-gray-400">Total Duration</span>
            <span className="font-mono font-semibold">{totalDuration.toFixed(2)}s</span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <span className="text-gray-400">Success Rate</span>
            <span className="font-mono font-semibold text-neon-green">
              {totalTests > 0 ? ((totalPassed / totalTests) * 100).toFixed(1) : 0}%
            </span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <span className="text-gray-400">Avg Test Duration</span>
            <span className="font-mono font-semibold">
              {totalTests > 0 ? (totalDuration / totalTests).toFixed(3) : 0}s
            </span>
          </div>
          <div className="flex items-center justify-between p-3 glass-card">
            <span className="text-gray-400">Test Suites</span>
            <span className="font-mono font-semibold">{testSuites.length}</span>
          </div>
        </div>
      </Card>
    </div>
  );
}
