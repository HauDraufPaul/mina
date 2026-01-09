import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { useErrorHandler } from "@/utils/errorHandler";
import { 
  Package, 
  RefreshCw, 
  AlertCircle, 
  CheckCircle, 
  Play, 
  Square,
  Search,
  X,
  Filter,
  TrendingUp,
  Server,
  Loader2
} from "lucide-react";

interface HomebrewPackage {
  name: string;
  version: string;
  installed: boolean;
  outdated: boolean;
  dependencies: string[];
  description?: string;
}

interface HomebrewService {
  name: string;
  status: string;
  running: boolean;
}

export default function PackagesRepository() {
  const errorHandler = useErrorHandler();
  const [packages, setPackages] = useState<HomebrewPackage[]>([]);
  const [services, setServices] = useState<HomebrewService[]>([]);
  const [outdated, setOutdated] = useState<string[]>([]);
  const [isAvailable, setIsAvailable] = useState(false);
  const [loading, setLoading] = useState(true);
  const [loadingPackages, setLoadingPackages] = useState(false);
  const [loadingOutdated, setLoadingOutdated] = useState(false);
  const [loadingServices, setLoadingServices] = useState(false);
  const [loadingDependencies, setLoadingDependencies] = useState(false);
  const [selectedPackage, setSelectedPackage] = useState<string | null>(null);
  const [dependencies, setDependencies] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterType, setFilterType] = useState<"all" | "outdated" | "installed">("all");
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState("");
  const itemsPerPage = 20;
  const abortControllerRef = useRef<AbortController | null>(null);
  const searchTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounce search query
  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current);
    }
    searchTimeoutRef.current = setTimeout(() => {
      setDebouncedSearchQuery(searchQuery);
      setCurrentPage(1); // Reset to first page on search
    }, 300);

    return () => {
      if (searchTimeoutRef.current) {
        clearTimeout(searchTimeoutRef.current);
      }
    };
  }, [searchQuery]);

  useEffect(() => {
    checkHomebrew();
    
    return () => {
      // Cancel any pending requests on unmount
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, []);

  const checkHomebrew = async () => {
    try {
      setLoading(true);
      setError(null);
      const available = await invoke<boolean>("is_homebrew_available");
      setIsAvailable(available);
      if (available) {
        // Progressive loading - load packages first (most important)
        await loadPackages();
        // Then load outdated and services in parallel (less critical)
        Promise.all([loadOutdated(), loadServices()]).catch(err => {
          errorHandler.showError("Failed to load some data", err);
        });
      }
    } catch (error) {
      errorHandler.showError("Failed to check Homebrew", error);
      setError(error instanceof Error ? error.message : "Failed to check Homebrew");
      setIsAvailable(false);
    } finally {
      setLoading(false);
    }
  };

  const loadPackages = async () => {
    // Cancel previous request if still pending
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    try {
      setLoadingPackages(true);
      setError(null);
      const pkgList = await invoke<HomebrewPackage[]>("list_installed_packages");
      if (!abortControllerRef.current?.signal.aborted) {
        setPackages(pkgList);
      }
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        return; // Request was cancelled, ignore
      }
      errorHandler.showError("Failed to load packages", error);
      if (!abortControllerRef.current?.signal.aborted) {
        setError(error instanceof Error ? error.message : "Failed to load packages");
      }
    } finally {
      if (!abortControllerRef.current?.signal.aborted) {
        setLoadingPackages(false);
      }
    }
  };

  const loadOutdated = async () => {
    try {
      setLoadingOutdated(true);
      const outdatedList = await invoke<string[]>("list_outdated_packages");
      setOutdated(outdatedList);
    } catch (error) {
      errorHandler.showError("Failed to load outdated packages", error);
    } finally {
      setLoadingOutdated(false);
    }
  };

  const loadServices = async () => {
    try {
      setLoadingServices(true);
      const serviceList = await invoke<HomebrewService[]>("list_services");
      setServices(serviceList);
    } catch (error) {
      errorHandler.showError("Failed to load services", error);
    } finally {
      setLoadingServices(false);
    }
  };

  const loadDependencies = useCallback(async (pkg: string) => {
    if (selectedPackage === pkg && dependencies.length > 0) {
      // Already loaded, just toggle
      setSelectedPackage(null);
      setDependencies([]);
      return;
    }

    try {
      setLoadingDependencies(true);
      const deps = await invoke<string[]>("get_package_dependencies", { package: pkg });
      setDependencies(deps);
      setSelectedPackage(pkg);
    } catch (error) {
      errorHandler.showError("Failed to load dependencies", error);
    } finally {
      setLoadingDependencies(false);
    }
  }, [selectedPackage, dependencies.length]);

  const handleServiceAction = async (service: string, action: "start" | "stop") => {
    try {
      // Optimistic update
      setServices(prev => prev.map(s => 
        s.name === service 
          ? { ...s, running: action === "start", status: action === "start" ? "started" : "stopped" }
          : s
      ));

      if (action === "start") {
        await invoke("start_service", { service });
      } else {
        await invoke("stop_service", { service });
      }
      // Refresh to get actual status
      await loadServices();
    } catch (error) {
      // Revert optimistic update on error
      await loadServices();
      errorHandler.showError(`Failed to ${action} service`, error);
    }
  };

  // Create a Set for O(1) lookup instead of O(n) array includes
  const outdatedSet = useMemo(() => new Set(outdated), [outdated]);

  // Optimized filtering with early returns and Set lookup
  const filteredPackages = useMemo(() => {
    if (packages.length === 0) return [];

    let filtered = packages;

    // Apply type filter first (usually faster)
    if (filterType === "outdated") {
      filtered = filtered.filter(pkg => outdatedSet.has(pkg.name));
    } else if (filterType === "installed") {
      filtered = filtered.filter(pkg => pkg.installed);
    }

    // Apply search filter (use debounced query for performance)
    if (debouncedSearchQuery.trim()) {
      const query = debouncedSearchQuery.toLowerCase();
      filtered = filtered.filter(pkg => {
        // Early return if name matches (most common case)
        if (pkg.name.toLowerCase().includes(query)) return true;
        if (pkg.version.toLowerCase().includes(query)) return true;
        if (pkg.description?.toLowerCase().includes(query)) return true;
        return false;
      });
    }

    return filtered;
  }, [packages, debouncedSearchQuery, filterType, outdatedSet]);

  const paginatedPackages = useMemo(() => {
    const start = (currentPage - 1) * itemsPerPage;
    return filteredPackages.slice(start, start + itemsPerPage);
  }, [filteredPackages, currentPage]);

  const totalPages = Math.ceil(filteredPackages.length / itemsPerPage);

  const handleRefresh = async () => {
    setCurrentPage(1);
    setSearchQuery("");
    setFilterType("all");
    await checkHomebrew();
  };

  // Memoized skeleton loader
  const PackageSkeleton = React.memo(() => (
    <div className="glass-card p-3 animate-pulse">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <div className="h-4 bg-white/10 rounded w-32 mb-2"></div>
          <div className="h-3 bg-white/5 rounded w-24"></div>
        </div>
        <div className="h-4 w-4 bg-white/10 rounded"></div>
      </div>
    </div>
  ));

  // Memoized package item component for better performance
  const PackageItem = React.memo(({ pkg, isSelected, onClick, isOutdated }: { 
    pkg: HomebrewPackage; 
    isSelected: boolean;
    onClick: () => void;
    isOutdated: boolean;
  }) => {
    return (
      <div
        className={`glass-card p-3 flex items-center justify-between hover:border-neon-cyan/50 transition-all cursor-pointer ${
          isSelected ? "border-neon-cyan border-2" : ""
        }`}
        onClick={onClick}
      >
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Package className="w-4 h-4 text-neon-cyan flex-shrink-0" />
            <span className="font-semibold truncate">{pkg.name}</span>
            {isOutdated && (
              <span className="text-xs px-2 py-0.5 bg-neon-amber/20 text-neon-amber rounded flex-shrink-0">
                Outdated
              </span>
            )}
          </div>
          <div className="text-xs text-gray-400 font-mono truncate">{pkg.version}</div>
          {pkg.description && (
            <div className="text-xs text-gray-500 mt-1 truncate">{pkg.description}</div>
          )}
        </div>
        <CheckCircle className="w-4 h-4 text-neon-green flex-shrink-0 ml-2" />
      </div>
    );
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <Loader2 className="w-8 h-8 mx-auto mb-4 text-neon-cyan animate-spin" />
          <p className="text-gray-400">Checking Homebrew availability...</p>
        </div>
      </div>
    );
  }

  if (!isAvailable) {
    return (
      <div className="space-y-6">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
            Packages Repository
          </h1>
          <p className="text-gray-400">Homebrew package management</p>
        </div>
        <Card title="Homebrew Not Available">
          <div className="text-center py-12">
            <AlertCircle className="w-16 h-16 mx-auto mb-4 text-neon-amber" />
            <p className="text-gray-300 mb-2 text-lg">Homebrew is not installed on this system.</p>
            <p className="text-sm text-gray-500 mb-6">
              Install Homebrew from{" "}
              <a
                href="https://brew.sh"
                target="_blank"
                rel="noopener noreferrer"
                className="text-neon-cyan hover:underline"
              >
                brew.sh
              </a>
            </p>
            <Button variant="primary" onClick={checkHomebrew}>
              <RefreshCw className="w-4 h-4 mr-2" />
              Check Again
            </Button>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-4xl font-bold mb-2 phosphor-glow-cyan">
            Packages Repository
          </h1>
          <p className="text-gray-400">Homebrew package management and monitoring</p>
        </div>
        <Button 
          onClick={handleRefresh} 
          variant="secondary"
          disabled={loadingPackages || loadingOutdated || loadingServices}
        >
          <RefreshCw className={`w-4 h-4 mr-2 ${loadingPackages ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      {/* Error Banner */}
      {error && (
        <Card className="border-neon-red/50 bg-neon-red/10">
          <div className="flex items-center gap-3">
            <AlertCircle className="w-5 h-5 text-neon-red" />
            <div className="flex-1">
              <p className="text-sm text-neon-red font-semibold">Error</p>
              <p className="text-xs text-gray-400">{error}</p>
            </div>
            <button
              onClick={() => setError(null)}
              className="text-gray-400 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </Card>
      )}

      {/* Overview Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="bg-gradient-to-br from-neon-cyan/20 to-transparent border-neon-cyan/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-cyan/20">
              <Package className="w-5 h-5 text-neon-cyan" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Installed Packages</div>
              <div className="text-2xl font-bold text-neon-cyan">
                {loadingPackages ? (
                  <Loader2 className="w-5 h-5 animate-spin inline" />
                ) : (
                  packages.length
                )}
              </div>
            </div>
          </div>
        </Card>
        <Card className="bg-gradient-to-br from-neon-amber/20 to-transparent border-neon-amber/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-amber/20">
              <TrendingUp className="w-5 h-5 text-neon-amber" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Outdated</div>
              <div className="text-2xl font-bold text-neon-amber">
                {loadingOutdated ? (
                  <Loader2 className="w-5 h-5 animate-spin inline" />
                ) : (
                  outdated.length
                )}
              </div>
            </div>
          </div>
        </Card>
        <Card className="bg-gradient-to-br from-neon-green/20 to-transparent border-neon-green/30">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-neon-green/20">
              <Server className="w-5 h-5 text-neon-green" />
            </div>
            <div>
              <div className="text-xs text-gray-400">Services</div>
              <div className="text-2xl font-bold text-neon-green">
                {loadingServices ? (
                  <Loader2 className="w-5 h-5 animate-spin inline" />
                ) : (
                  services.length
                )}
              </div>
            </div>
          </div>
        </Card>
      </div>

      {/* Packages Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Installed Packages">
          {/* Search and Filter */}
          <div className="mb-4 space-y-3">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => {
                  setSearchQuery(e.target.value);
                }}
                placeholder="Search packages..."
                className="glass-input w-full pl-10 pr-10"
              />
              {searchQuery !== debouncedSearchQuery && (
                <div className="absolute right-10 top-1/2 transform -translate-y-1/2">
                  <Loader2 className="w-3 h-3 text-gray-400 animate-spin" />
                </div>
              )}
              {searchQuery && (
                <button
                  onClick={() => {
                    setSearchQuery("");
                    setCurrentPage(1);
                  }}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white"
                >
                  <X className="w-4 h-4" />
                </button>
              )}
            </div>
            <div className="flex items-center gap-2">
              <Filter className="w-4 h-4 text-gray-400" />
              <select
                value={filterType}
                onChange={(e) => {
                  setFilterType(e.target.value as "all" | "outdated" | "installed");
                  setCurrentPage(1);
                }}
                className="glass-input flex-1"
              >
                <option value="all">All Packages</option>
                <option value="outdated">Outdated Only</option>
                <option value="installed">Installed Only</option>
              </select>
            </div>
          </div>

          {/* Package List */}
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {loadingPackages ? (
              Array.from({ length: 5 }).map((_, i) => <PackageSkeleton key={i} />)
            ) : paginatedPackages.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <Package className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>
                  {searchQuery || filterType !== "all"
                    ? "No packages found matching your criteria"
                    : "No packages installed"}
                </p>
              </div>
            ) : (
              paginatedPackages.map((pkg) => (
                <PackageItem
                  key={pkg.name}
                  pkg={pkg}
                  isSelected={selectedPackage === pkg.name}
                  isOutdated={outdatedSet.has(pkg.name)}
                  onClick={() => loadDependencies(pkg.name)}
                />
              ))
            )}
          </div>

          {/* Pagination */}
          {!loadingPackages && filteredPackages.length > itemsPerPage && (
            <div className="mt-4 flex items-center justify-between pt-4 border-t border-white/10">
              <div className="text-xs text-gray-400">
                Showing {((currentPage - 1) * itemsPerPage) + 1} - {Math.min(currentPage * itemsPerPage, filteredPackages.length)} of {filteredPackages.length}
              </div>
              <div className="flex gap-2">
                <Button
                  variant="secondary"
                  onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                  disabled={currentPage === 1}
                  className="text-xs"
                >
                  Previous
                </Button>
                <Button
                  variant="secondary"
                  onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                  disabled={currentPage === totalPages}
                  className="text-xs"
                >
                  Next
                </Button>
              </div>
            </div>
          )}
        </Card>

        {/* Services Section */}
        <Card title="Services">
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {loadingServices ? (
              Array.from({ length: 3 }).map((_, i) => <PackageSkeleton key={i} />)
            ) : services.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <Server className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>No services found</p>
              </div>
            ) : (
              services.map((service) => (
                <div
                  key={service.name}
                  className="glass-card p-3 flex items-center justify-between"
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-semibold truncate">{service.name}</span>
                      <span
                        className={`text-xs px-2 py-0.5 rounded flex-shrink-0 ${
                          service.running
                            ? "bg-neon-green/20 text-neon-green"
                            : "bg-gray-800 text-gray-400"
                        }`}
                      >
                        {service.status}
                      </span>
                    </div>
                  </div>
                  <div className="flex gap-2 flex-shrink-0 ml-2">
                    {service.running ? (
                      <Button
                        onClick={() => handleServiceAction(service.name, "stop")}
                        variant="ghost"
                        className="p-1"
                        title="Stop service"
                      >
                        <Square className="w-4 h-4 text-neon-red" />
                      </Button>
                    ) : (
                      <Button
                        onClick={() => handleServiceAction(service.name, "start")}
                        variant="ghost"
                        className="p-1"
                        title="Start service"
                      >
                        <Play className="w-4 h-4 text-neon-green" />
                      </Button>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </Card>
      </div>

      {/* Dependencies Section */}
      {selectedPackage && (
        <Card title={`Dependencies: ${selectedPackage}`}>
          {loadingDependencies ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 text-neon-cyan animate-spin mr-2" />
              <span className="text-gray-400">Loading dependencies...</span>
            </div>
          ) : dependencies.length === 0 ? (
            <div className="text-center py-8 text-gray-400">
              <CheckCircle className="w-12 h-12 mx-auto mb-4 text-neon-green" />
              <p>No dependencies</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
              {dependencies.map((dep) => (
                <div key={dep} className="glass-card p-2 flex items-center gap-2">
                  <Package className="w-4 h-4 text-neon-cyan flex-shrink-0" />
                  <span className="font-mono text-sm truncate">{dep}</span>
                </div>
              ))}
            </div>
          )}
        </Card>
      )}

      {/* Outdated Packages Section */}
      <Card title="Outdated Packages">
        {loadingOutdated ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-6 h-6 text-neon-amber animate-spin mr-2" />
            <span className="text-gray-400">Checking for outdated packages...</span>
          </div>
        ) : outdated.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            <CheckCircle className="w-12 h-12 mx-auto mb-4 text-neon-green" />
            <p className="text-lg mb-2">All packages are up to date!</p>
            <p className="text-sm text-gray-500">Your system is running the latest versions</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {outdated.map((pkg) => (
              <div key={pkg} className="glass-card p-3 flex items-center justify-between border-l-4 border-neon-amber">
                <div className="flex items-center gap-2 min-w-0">
                  <AlertCircle className="w-4 h-4 text-neon-amber flex-shrink-0" />
                  <span className="font-semibold truncate">{pkg}</span>
                </div>
                <Button variant="secondary" className="text-xs flex-shrink-0 ml-2">
                  Update
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
