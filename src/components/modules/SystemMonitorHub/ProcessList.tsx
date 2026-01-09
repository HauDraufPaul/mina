import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Trash2, Activity, Search, ArrowUpDown, ArrowUp, ArrowDown, X } from "lucide-react";
import { useErrorHandler } from "@/utils/errorHandler";

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  status: string;
  parent_pid?: number;
}

type SortField = "pid" | "name" | "cpu_usage" | "memory_usage" | "status";
type SortDirection = "asc" | "desc";

export default function ProcessList() {
  const errorHandler = useErrorHandler();
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [filteredProcesses, setFilteredProcesses] = useState<ProcessInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [sortField, setSortField] = useState<SortField>("cpu_usage");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(20);

  useEffect(() => {
    const fetchProcesses = async () => {
      try {
        const data = await invoke<ProcessInfo[]>("get_processes");
        setProcesses(data);
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

  // Filter and sort processes
  useEffect(() => {
    let filtered = [...processes];

    // Apply search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (p) =>
          p.name.toLowerCase().includes(query) ||
          p.pid.toString().includes(query) ||
          p.status.toLowerCase().includes(query)
      );
    }

    // Apply sorting
    filtered.sort((a, b) => {
      let aVal: number | string;
      let bVal: number | string;

      switch (sortField) {
        case "pid":
          aVal = a.pid;
          bVal = b.pid;
          break;
        case "name":
          aVal = a.name.toLowerCase();
          bVal = b.name.toLowerCase();
          break;
        case "cpu_usage":
          aVal = a.cpu_usage;
          bVal = b.cpu_usage;
          break;
        case "memory_usage":
          aVal = a.memory_usage;
          bVal = b.memory_usage;
          break;
        case "status":
          aVal = a.status.toLowerCase();
          bVal = b.status.toLowerCase();
          break;
        default:
          return 0;
      }

      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortDirection === "asc" ? aVal - bVal : bVal - aVal;
      } else {
        return sortDirection === "asc"
          ? String(aVal).localeCompare(String(bVal))
          : String(bVal).localeCompare(String(aVal));
      }
    });

    setFilteredProcesses(filtered);
    setCurrentPage(1); // Reset to first page when filter changes
  }, [processes, searchQuery, sortField, sortDirection]);

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDirection("desc");
    }
  };

  const handleKillProcess = async (pid: number, name: string) => {
    if (!confirm(`Are you sure you want to kill process "${name}" (PID: ${pid})?`)) {
      return;
    }

    try {
      await invoke("kill_process", { pid });
      // Refresh processes
      const data = await invoke<ProcessInfo[]>("get_processes");
      setProcesses(data);
      errorHandler.showSuccess("Process killed successfully");
    } catch (error) {
      errorHandler.showError("Failed to kill process", error);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const getCpuColor = (usage: number) => {
    if (usage >= 50) return "text-neon-red";
    if (usage >= 20) return "text-neon-amber";
    return "text-neon-green";
  };

  const getMemoryColor = (usage: number, total: number) => {
    const percent = (usage / total) * 100;
    if (percent >= 5) return "text-neon-red";
    if (percent >= 2) return "text-neon-amber";
    return "text-neon-green";
  };

  // Pagination
  const totalPages = Math.ceil(filteredProcesses.length / itemsPerPage);
  const paginatedProcesses = useMemo(() => {
    const start = (currentPage - 1) * itemsPerPage;
    return filteredProcesses.slice(start, start + itemsPerPage);
  }, [filteredProcesses, currentPage, itemsPerPage]);

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) {
      return <ArrowUpDown className="w-3 h-3 text-gray-500" />;
    }
    return sortDirection === "asc" ? (
      <ArrowUp className="w-3 h-3 text-neon-cyan" />
    ) : (
      <ArrowDown className="w-3 h-3 text-neon-cyan" />
    );
  };

  if (loading) {
    return (
      <Card title="Running Processes">
        <div className="text-center py-8">
          <Activity className="w-8 h-8 mx-auto mb-4 text-neon-cyan animate-pulse" />
          <p className="text-gray-400">Loading processes...</p>
        </div>
      </Card>
    );
  }

  return (
    <Card
      title="Running Processes"
      subtitle={`${filteredProcesses.length} of ${processes.length} processes`}
    >
      {/* Search and Controls */}
      <div className="mb-4 flex items-center gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search by name, PID, or status..."
            className="glass-input w-full pl-10 pr-10"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      {/* Process Table */}
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-white/10">
              <th
                className="text-left p-3 text-xs font-semibold text-gray-400 cursor-pointer hover:text-gray-300 transition-colors"
                onClick={() => handleSort("pid")}
              >
                <div className="flex items-center gap-2">
                  PID
                  <SortIcon field="pid" />
                </div>
              </th>
              <th
                className="text-left p-3 text-xs font-semibold text-gray-400 cursor-pointer hover:text-gray-300 transition-colors"
                onClick={() => handleSort("name")}
              >
                <div className="flex items-center gap-2">
                  Name
                  <SortIcon field="name" />
                </div>
              </th>
              <th
                className="text-right p-3 text-xs font-semibold text-gray-400 cursor-pointer hover:text-gray-300 transition-colors"
                onClick={() => handleSort("cpu_usage")}
              >
                <div className="flex items-center justify-end gap-2">
                  CPU %
                  <SortIcon field="cpu_usage" />
                </div>
              </th>
              <th
                className="text-right p-3 text-xs font-semibold text-gray-400 cursor-pointer hover:text-gray-300 transition-colors"
                onClick={() => handleSort("memory_usage")}
              >
                <div className="flex items-center justify-end gap-2">
                  Memory
                  <SortIcon field="memory_usage" />
                </div>
              </th>
              <th
                className="text-left p-3 text-xs font-semibold text-gray-400 cursor-pointer hover:text-gray-300 transition-colors"
                onClick={() => handleSort("status")}
              >
                <div className="flex items-center gap-2">
                  Status
                  <SortIcon field="status" />
                </div>
              </th>
              <th className="text-center p-3 text-xs font-semibold text-gray-400">
                Actions
              </th>
            </tr>
          </thead>
          <tbody>
            {paginatedProcesses.length === 0 ? (
              <tr>
                <td colSpan={6} className="text-center py-8 text-gray-400">
                  {searchQuery ? "No processes found matching your search" : "No processes found"}
                </td>
              </tr>
            ) : (
              paginatedProcesses.map((process) => (
                <tr
                  key={process.pid}
                  className="border-b border-white/5 hover:bg-white/5 transition-colors"
                >
                  <td className="p-3 text-sm font-mono text-gray-300">{process.pid}</td>
                  <td className="p-3 text-sm text-gray-200 max-w-xs truncate" title={process.name}>
                    {process.name}
                  </td>
                  <td className="p-3 text-sm text-right">
                    <div className="flex items-center justify-end gap-2">
                      <Activity className={`w-3 h-3 ${getCpuColor(process.cpu_usage)}`} />
                      <span className={`font-semibold ${getCpuColor(process.cpu_usage)}`}>
                        {process.cpu_usage.toFixed(1)}%
                      </span>
                    </div>
                  </td>
                  <td className="p-3 text-sm text-right font-mono">
                    <span className={getMemoryColor(process.memory_usage, 16 * 1024 * 1024 * 1024)}>
                      {formatBytes(process.memory_usage)}
                    </span>
                  </td>
                  <td className="p-3 text-sm">
                    <span
                      className={`px-2 py-1 rounded text-xs ${
                        process.status === "Running"
                          ? "bg-neon-green/20 text-neon-green"
                          : process.status === "Sleeping"
                          ? "bg-neon-amber/20 text-neon-amber"
                          : "bg-gray-500/20 text-gray-400"
                      }`}
                    >
                      {process.status}
                    </span>
                  </td>
                  <td className="p-3 text-center">
                    <Button
                      variant="ghost"
                      className="p-1 hover:bg-neon-red/20 transition-colors"
                      onClick={() => handleKillProcess(process.pid, process.name)}
                      title={`Kill process ${process.pid}`}
                    >
                      <Trash2 className="w-4 h-4 text-neon-red" />
                    </Button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-between">
          <div className="text-sm text-gray-400">
            Showing {(currentPage - 1) * itemsPerPage + 1} to{" "}
            {Math.min(currentPage * itemsPerPage, filteredProcesses.length)} of{" "}
            {filteredProcesses.length} processes
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
              disabled={currentPage === 1}
            >
              Previous
            </Button>
            <span className="text-sm text-gray-400 px-4">
              Page {currentPage} of {totalPages}
            </span>
            <Button
              variant="secondary"
              onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </Card>
  );
}
