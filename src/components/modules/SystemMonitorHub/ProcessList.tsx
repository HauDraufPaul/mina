import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Trash2, Activity } from "lucide-react";

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  status: string;
  parent_pid?: number;
}

export default function ProcessList() {
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchProcesses = async () => {
      try {
        const data = await invoke<ProcessInfo[]>("get_processes");
        setProcesses(data.slice(0, 20)); // Show top 20
        setLoading(false);
      } catch (error) {
        console.error("Failed to fetch processes:", error);
        setLoading(false);
      }
    };

    fetchProcesses();
    const interval = setInterval(fetchProcesses, 2000);

    return () => clearInterval(interval);
  }, []);

  const handleKillProcess = async (pid: number) => {
    if (!confirm(`Are you sure you want to kill process ${pid}?`)) {
      return;
    }

    try {
      await invoke("kill_process", { pid });
      // Refresh processes
      const data = await invoke<ProcessInfo[]>("get_processes");
      setProcesses(data.slice(0, 20));
    } catch (error) {
      alert(`Failed to kill process: ${error}`);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  if (loading) {
    return <div className="text-center">Loading processes...</div>;
  }

  return (
    <Card title="Running Processes" subtitle="Top processes by CPU usage">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-white/10">
              <th className="text-left p-2 text-sm text-gray-400">PID</th>
              <th className="text-left p-2 text-sm text-gray-400">Name</th>
              <th className="text-right p-2 text-sm text-gray-400">CPU %</th>
              <th className="text-right p-2 text-sm text-gray-400">Memory</th>
              <th className="text-left p-2 text-sm text-gray-400">Status</th>
              <th className="text-center p-2 text-sm text-gray-400">Actions</th>
            </tr>
          </thead>
          <tbody>
            {processes.map((process) => (
              <tr
                key={process.pid}
                className="border-b border-white/5 hover:bg-white/5"
              >
                <td className="p-2 text-sm font-mono">{process.pid}</td>
                <td className="p-2 text-sm">{process.name}</td>
                <td className="p-2 text-sm text-right">
                  <div className="flex items-center justify-end gap-2">
                    <Activity className="w-3 h-3 text-neon-cyan" />
                    {process.cpu_usage.toFixed(1)}%
                  </div>
                </td>
                <td className="p-2 text-sm text-right font-mono">
                  {formatBytes(process.memory_usage)}
                </td>
                <td className="p-2 text-sm text-gray-400">{process.status}</td>
                <td className="p-2 text-center">
                  <Button
                    variant="ghost"
                    className="p-1"
                    onClick={() => handleKillProcess(process.pid)}
                  >
                    <Trash2 className="w-4 h-4 text-neon-red" />
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Card>
  );
}

