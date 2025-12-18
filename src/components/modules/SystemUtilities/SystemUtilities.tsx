import { useState } from "react";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { HardDrive, Power, Settings, Activity } from "lucide-react";

export default function SystemUtilities() {
  const [diskInfo, setDiskInfo] = useState({
    total: 0,
    used: 0,
    free: 0,
  });

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          System Utilities
        </h1>
        <p className="text-gray-400">System management and diagnostics</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card title="Disk Management" subtitle="Storage analysis and cleanup">
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <HardDrive className="w-12 h-12 text-neon-cyan" />
              <div className="flex-1">
                <div className="text-sm text-gray-400 mb-1">Total Space</div>
                <div className="text-lg font-semibold">
                  {(diskInfo.total / 1024 / 1024 / 1024).toFixed(2)} GB
                </div>
              </div>
            </div>
            <div className="w-full bg-gray-800 rounded-full h-2">
              <div
                className="bg-neon-cyan h-2 rounded-full"
                style={{
                  width: `${diskInfo.total > 0 ? (diskInfo.used / diskInfo.total) * 100 : 0}%`,
                }}
              />
            </div>
            <Button variant="secondary" className="w-full">
              Analyze Disk Usage
            </Button>
            <Button variant="secondary" className="w-full">
              Clean Cache
            </Button>
          </div>
        </Card>

        <Card title="Power Management" subtitle="System power controls">
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <Power className="w-12 h-12 text-neon-green" />
              <div className="flex-1">
                <div className="text-sm text-gray-400 mb-1">Power Status</div>
                <div className="text-lg font-semibold text-neon-green">
                  Active
                </div>
              </div>
            </div>
            <div className="space-y-2">
              <Button variant="secondary" className="w-full">
                Prevent Sleep
              </Button>
              <Button variant="secondary" className="w-full">
                Display Controls
              </Button>
            </div>
          </div>
        </Card>

        <Card title="Service Control" subtitle="System service management">
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <Settings className="w-12 h-12 text-neon-amber" />
              <div className="flex-1">
                <div className="text-sm text-gray-400 mb-1">Services</div>
                <div className="text-lg font-semibold">Manage Services</div>
              </div>
            </div>
            <Button variant="secondary" className="w-full">
              View Services
            </Button>
            <Button variant="secondary" className="w-full">
              Service Status
            </Button>
          </div>
        </Card>

        <Card title="Hardware Diagnostics" subtitle="System health checks">
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <Activity className="w-12 h-12 text-neon-red" />
              <div className="flex-1">
                <div className="text-sm text-gray-400 mb-1">Health Status</div>
                <div className="text-lg font-semibold text-neon-green">
                  Healthy
                </div>
              </div>
            </div>
            <Button variant="secondary" className="w-full">
              Run Diagnostics
            </Button>
            <Button variant="secondary" className="w-full">
              View Reports
            </Button>
          </div>
        </Card>
      </div>

      <Card title="System Information">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <div className="text-sm text-gray-400 mb-1">OS</div>
            <div className="font-mono text-sm">macOS</div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">Architecture</div>
            <div className="font-mono text-sm">ARM64</div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">Kernel</div>
            <div className="font-mono text-sm">Darwin</div>
          </div>
          <div>
            <div className="text-sm text-gray-400 mb-1">Uptime</div>
            <div className="font-mono text-sm">24h 15m</div>
          </div>
        </div>
      </Card>
    </div>
  );
}
