import { useState } from "react";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { TestTube, Play, CheckCircle, XCircle, Clock, BarChart3 } from "lucide-react";

interface TestResult {
  name: string;
  status: "passed" | "failed" | "running" | "pending";
  duration?: number;
  error?: string;
}

interface TestSuite {
  name: string;
  tests: TestResult[];
  total: number;
  passed: number;
  failed: number;
  duration: number;
}

export default function TestingCenter() {
  const [testSuites, setTestSuites] = useState<TestSuite[]>([]);
  const [running, setRunning] = useState(false);
  const [selectedSuite, setSelectedSuite] = useState<string | null>(null);

  const mockTestSuites: TestSuite[] = [
    {
      name: "Unit Tests",
      total: 45,
      passed: 42,
      failed: 3,
      duration: 2.3,
      tests: [
        { name: "SystemProvider::get_cpu_usage", status: "passed", duration: 0.05 },
        { name: "Database::save_error", status: "passed", duration: 0.12 },
        { name: "AuthManager::verify_pin", status: "passed", duration: 0.08 },
        { name: "VectorStore::search_similar", status: "failed", duration: 0.15, error: "Assertion failed: similarity > 0.9" },
        { name: "HomebrewProvider::list_installed", status: "passed", duration: 0.23 },
      ],
    },
    {
      name: "Integration Tests",
      total: 12,
      passed: 11,
      failed: 1,
      duration: 5.7,
      tests: [
        { name: "System metrics collection", status: "passed", duration: 1.2 },
        { name: "Network interface monitoring", status: "passed", duration: 0.8 },
        { name: "Process management", status: "passed", duration: 0.5 },
        { name: "Database migrations", status: "failed", duration: 2.1, error: "Migration rollback failed" },
      ],
    },
    {
      name: "E2E Tests",
      total: 8,
      passed: 7,
      failed: 1,
      duration: 12.4,
      tests: [
        { name: "User login flow", status: "passed", duration: 3.2 },
        { name: "System monitor display", status: "passed", duration: 2.8 },
        { name: "Package installation", status: "passed", duration: 4.1 },
        { name: "Vector search workflow", status: "failed", duration: 2.3, error: "Timeout waiting for results" },
      ],
    },
  ];

  const handleRunTests = async (suiteName?: string) => {
    setRunning(true);
    // Simulate test execution
    setTimeout(() => {
      if (suiteName) {
        setTestSuites(mockTestSuites.filter((s) => s.name === suiteName));
      } else {
        setTestSuites(mockTestSuites);
      }
      setRunning(false);
    }, 2000);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
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

  const totalTests = testSuites.reduce((sum, suite) => sum + suite.total, 0);
  const totalPassed = testSuites.reduce((sum, suite) => sum + suite.passed, 0);
  const totalFailed = testSuites.reduce((sum, suite) => sum + suite.failed, 0);
  const totalDuration = testSuites.reduce((sum, suite) => sum + suite.duration, 0);
  const coverage = totalTests > 0 ? (totalPassed / totalTests) * 100 : 0;

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
          <Button onClick={() => handleRunTests()} variant="primary" disabled={running}>
            <Play className="w-4 h-4 mr-2" />
            Run All Tests
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
        {testSuites.map((suite) => (
          <Card key={suite.name} title={suite.name}>
            <div className="space-y-4">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <TestTube className="w-5 h-5 text-neon-cyan" />
                  <span className="font-semibold">{suite.name}</span>
                </div>
                <div className="text-sm text-gray-400">
                  {suite.duration.toFixed(2)}s
                </div>
              </div>

              <div className="grid grid-cols-3 gap-2 mb-4">
                <div className="text-center">
                  <div className="text-lg font-bold text-neon-cyan">{suite.total}</div>
                  <div className="text-xs text-gray-400">Total</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-bold text-neon-green">{suite.passed}</div>
                  <div className="text-xs text-gray-400">Passed</div>
                </div>
                <div className="text-center">
                  <div className="text-lg font-bold text-neon-red">{suite.failed}</div>
                  <div className="text-xs text-gray-400">Failed</div>
                </div>
              </div>

              <div className="w-full bg-gray-800 rounded-full h-2 mb-4">
                <div
                  className="bg-neon-green h-2 rounded-full transition-all"
                  style={{ width: `${(suite.passed / suite.total) * 100}%` }}
                />
              </div>

              <div className="space-y-2 max-h-64 overflow-y-auto">
                {suite.tests.map((test, index) => (
                  <div
                    key={index}
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
                ))}
              </div>

              <Button
                onClick={() => handleRunTests(suite.name)}
                variant="secondary"
                className="w-full"
                disabled={running}
              >
                <Play className="w-4 h-4 mr-2" />
                Run {suite.name}
              </Button>
            </div>
          </Card>
        ))}
      </div>

      {testSuites.length === 0 && (
        <Card title="No Test Results">
          <div className="text-center py-8 text-gray-400">
            <TestTube className="w-12 h-12 mx-auto mb-4 text-gray-500" />
            <p>Click "Run All Tests" to execute the test suite</p>
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
