import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { HardDrive, Power, Settings, Activity } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface DiskInfo {
  total: number;
  used: number;
  free: number;
  usage_percent: number;
}

interface SystemInfo {
  os: string;
  architecture: string;
  kernel: string;
  uptime: number;
}

export default function SystemUtilities() {
  const errorHandler = useErrorHandler();
  const [diskInfo, setDiskInfo] = useState<DiskInfo | null>(null);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadData = async () => {
    try {
      const [disk, system] = await Promise.all([
        invoke<DiskInfo>("get_disk_info"),
        invoke<SystemInfo>("get_system_info"),
      ]);
      console.log("System utilities data loaded:", { disk, system });
      setDiskInfo(disk);
      setSystemInfo(system);
      setLoading(false);
    } catch (error) {
      errorHandler.showError("Failed to load system data", error);
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h ${minutes}m`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const handlePreventSleep = async () => {
    try {
      await invoke("prevent_sleep");
      errorHandler.showSuccess("Sleep prevention activated");
    } catch (error) {
      errorHandler.showError("Failed to prevent sleep", error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          System Utilities
        </h1>
        <p className="text-gray-400">System management and diagnostics</p>
      </div>

      {loading ? (
        <div className="text-center">Loading system information...</div>
      ) : (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card title="Disk Management" subtitle="Storage analysis and cleanup">
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <HardDrive className="w-12 h-12 text-neon-cyan" />
                  <div className="flex-1">
                    <div className="text-sm text-gray-400 mb-1">Total Space</div>
                    <div className="text-lg font-semibold">
                      {diskInfo ? formatBytes(diskInfo.total) : "N/A"}
                    </div>
                  </div>
                </div>
                {diskInfo && (
                  <>
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-400">Used</span>
                        <span>{formatBytes(diskInfo.used)}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-400">Free</span>
                        <span>{formatBytes(diskInfo.free)}</span>
                      </div>
                    </div>
                    <div className="w-full bg-gray-800 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full transition-all ${
                          diskInfo.usage_percent > 90
                            ? "bg-neon-red"
                            : diskInfo.usage_percent > 70
                            ? "bg-neon-amber"
                            : "bg-neon-cyan"
                        }`}
                        style={{ width: `${diskInfo.usage_percent}%` }}
                      />
                    </div>
                    <div className="text-xs text-gray-400 text-center">
                      {diskInfo.usage_percent.toFixed(1)}% used
                    </div>
                  </>
                )}
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
                  <Button
                    variant="secondary"
                    className="w-full"
                    onClick={handlePreventSleep}
                  >
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
            {systemInfo ? (
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <div className="text-sm text-gray-400 mb-1">OS</div>
                  <div className="font-mono text-sm">{systemInfo.os}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1">Architecture</div>
                  <div className="font-mono text-sm">{systemInfo.architecture}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1">Kernel</div>
                  <div className="font-mono text-sm">{systemInfo.kernel}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400 mb-1">Uptime</div>
                  <div className="font-mono text-sm">
                    {formatUptime(systemInfo.uptime)}
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-center text-gray-400 py-4">
                System information not available
              </div>
            )}
          </Card>
        </>
      )}
    </div>
  );
}
