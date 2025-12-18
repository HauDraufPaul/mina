import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Package, RefreshCw, AlertCircle, CheckCircle, Play, Square } from "lucide-react";

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
  const [packages, setPackages] = useState<HomebrewPackage[]>([]);
  const [services, setServices] = useState<HomebrewService[]>([]);
  const [outdated, setOutdated] = useState<string[]>([]);
  const [isAvailable, setIsAvailable] = useState(false);
  const [loading, setLoading] = useState(true);
  const [selectedPackage, setSelectedPackage] = useState<string | null>(null);
  const [dependencies, setDependencies] = useState<string[]>([]);

  useEffect(() => {
    checkHomebrew();
  }, []);

  const checkHomebrew = async () => {
    try {
      const available = await invoke<boolean>("is_homebrew_available");
      setIsAvailable(available);
      if (available) {
        await loadData();
      }
      setLoading(false);
    } catch (error) {
      console.error("Failed to check Homebrew:", error);
      setIsAvailable(false);
      setLoading(false);
    }
  };

  const loadData = async () => {
    try {
      const [pkgList, outdatedList, serviceList] = await Promise.all([
        invoke<HomebrewPackage[]>("list_installed_packages"),
        invoke<string[]>("list_outdated_packages"),
        invoke<HomebrewService[]>("list_services"),
      ]);
      setPackages(pkgList);
      setOutdated(outdatedList);
      setServices(serviceList);
    } catch (error) {
      console.error("Failed to load data:", error);
    }
  };

  const loadDependencies = async (pkg: string) => {
    try {
      const deps = await invoke<string[]>("get_package_dependencies", { package: pkg });
      setDependencies(deps);
      setSelectedPackage(pkg);
    } catch (error) {
      console.error("Failed to load dependencies:", error);
    }
  };

  const handleServiceAction = async (service: string, action: "start" | "stop") => {
    try {
      if (action === "start") {
        await invoke("start_service", { service });
      } else {
        await invoke("stop_service", { service });
      }
      await loadData();
    } catch (error) {
      alert(`Failed to ${action} service: ${error}`);
    }
  };

  if (loading) {
    return <div className="text-center">Checking Homebrew availability...</div>;
  }

  if (!isAvailable) {
    return (
      <div className="space-y-6">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Packages Repository
          </h1>
          <p className="text-gray-400">Homebrew package management</p>
        </div>
        <Card title="Homebrew Not Available">
          <div className="text-center py-8">
            <AlertCircle className="w-12 h-12 mx-auto mb-4 text-neon-amber" />
            <p className="text-gray-400 mb-4">
              Homebrew is not installed on this system.
            </p>
            <p className="text-sm text-gray-500">
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
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Packages Repository
          </h1>
          <p className="text-gray-400">Homebrew package management</p>
        </div>
        <Button onClick={loadData} variant="secondary">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <Card title="Installed Packages" subtitle="Total packages">
          <div className="text-3xl font-bold text-neon-cyan">{packages.length}</div>
        </Card>
        <Card title="Outdated" subtitle="Packages needing update">
          <div className="text-3xl font-bold text-neon-amber">{outdated.length}</div>
        </Card>
        <Card title="Services" subtitle="Homebrew services">
          <div className="text-3xl font-bold text-neon-green">{services.length}</div>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card title="Installed Packages">
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {packages.map((pkg) => (
              <div
                key={pkg.name}
                className="glass-card p-3 flex items-center justify-between hover:border-neon-cyan transition-all cursor-pointer"
                onClick={() => loadDependencies(pkg.name)}
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <Package className="w-4 h-4 text-neon-cyan" />
                    <span className="font-semibold">{pkg.name}</span>
                    {outdated.includes(pkg.name) && (
                      <span className="text-xs px-2 py-0.5 bg-neon-amber/20 text-neon-amber rounded">
                        Outdated
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-gray-400 font-mono">{pkg.version}</div>
                </div>
                <CheckCircle className="w-4 h-4 text-neon-green" />
              </div>
            ))}
          </div>
        </Card>

        <Card title="Services">
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {services.map((service) => (
              <div
                key={service.name}
                className="glass-card p-3 flex items-center justify-between"
              >
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-semibold">{service.name}</span>
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        service.running
                          ? "bg-neon-green/20 text-neon-green"
                          : "bg-gray-800 text-gray-400"
                      }`}
                    >
                      {service.status}
                    </span>
                  </div>
                </div>
                <div className="flex gap-2">
                  {service.running ? (
                    <Button
                      onClick={() => handleServiceAction(service.name, "stop")}
                      variant="ghost"
                      className="p-1"
                    >
                      <Square className="w-4 h-4 text-neon-red" />
                    </Button>
                  ) : (
                    <Button
                      onClick={() => handleServiceAction(service.name, "start")}
                      variant="ghost"
                      className="p-1"
                    >
                      <Play className="w-4 h-4 text-neon-green" />
                    </Button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>

      {selectedPackage && (
        <Card title={`Dependencies: ${selectedPackage}`}>
          <div className="space-y-2">
            {dependencies.length === 0 ? (
              <p className="text-gray-400">No dependencies</p>
            ) : (
              dependencies.map((dep) => (
                <div key={dep} className="glass-card p-2 flex items-center gap-2">
                  <Package className="w-4 h-4 text-neon-cyan" />
                  <span className="font-mono text-sm">{dep}</span>
                </div>
              ))
            )}
          </div>
        </Card>
      )}

      <Card title="Outdated Packages">
        {outdated.length === 0 ? (
          <div className="text-center text-gray-400 py-8">
            <CheckCircle className="w-12 h-12 mx-auto mb-4 text-neon-green" />
            <p>All packages are up to date!</p>
          </div>
        ) : (
          <div className="space-y-2">
            {outdated.map((pkg) => (
              <div key={pkg} className="glass-card p-3 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <AlertCircle className="w-4 h-4 text-neon-amber" />
                  <span className="font-semibold">{pkg}</span>
                </div>
                <Button variant="secondary" className="text-xs">
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
