import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Save, RefreshCw, Settings } from "lucide-react";

interface ConfigEntry {
  key: string;
  value: string;
}

export default function ConfigurationManager() {
  const [configs, setConfigs] = useState<ConfigEntry[]>([]);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const [loading, setLoading] = useState(false);

  const loadConfigs = async () => {
    setLoading(true);
    try {
      // Load common configuration keys
      const keys = [
        "ws_addr",
        "log_level",
        "neo4j_uri",
        "neo4j_user",
        "neo4j_password",
        "openai_api_key",
        "anthropic_api_key",
      ];

      const loaded: ConfigEntry[] = [];
      for (const key of keys) {
        try {
          const value = await invoke<string | null>("get_config", { key });
          if (value !== null) {
            loaded.push({ key, value });
          } else {
            loaded.push({ key, value: "" });
          }
        } catch (error) {
          console.error(`Failed to load config ${key}:`, error);
          loaded.push({ key, value: "" });
        }
      }
      setConfigs(loaded);
    } catch (error) {
      console.error("Failed to load configs:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadConfigs();
  }, []);

  const handleSave = async (key: string, value: string) => {
    try {
      await invoke("set_config", { key, value });
      await loadConfigs();
    } catch (error) {
      alert(`Failed to save config: ${error}`);
    }
  };

  const handleAdd = async () => {
    if (!newKey.trim()) {
      alert("Key cannot be empty");
      return;
    }
    try {
      await invoke("set_config", { key: newKey, value: newValue });
      setNewKey("");
      setNewValue("");
      await loadConfigs();
    } catch (error) {
      alert(`Failed to add config: ${error}`);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
          Configuration Manager
        </h1>
        <p className="text-gray-400">Application configuration and settings</p>
      </div>

      <div className="flex justify-end mb-4">
        <Button onClick={loadConfigs} variant="secondary">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      <Card title="Configuration Entries">
        {loading ? (
          <div className="text-center py-8">Loading configuration...</div>
        ) : (
          <div className="space-y-4">
            {configs.map((config) => (
              <ConfigRow
                key={config.key}
                entry={config}
                onSave={handleSave}
              />
            ))}
          </div>
        )}
      </Card>

      <Card title="Add New Configuration">
        <div className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Key</label>
            <input
              type="text"
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              className="glass-input w-full"
              placeholder="config_key"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-2">Value</label>
            <input
              type="text"
              value={newValue}
              onChange={(e) => setNewValue(e.target.value)}
              className="glass-input w-full"
              placeholder="config_value"
            />
          </div>
          <Button onClick={handleAdd} variant="primary">
            <Save className="w-4 h-4 mr-2" />
            Add Configuration
          </Button>
        </div>
      </Card>
    </div>
  );
}

function ConfigRow({
  entry,
  onSave,
}: {
  entry: ConfigEntry;
  onSave: (key: string, value: string) => void;
}) {
  const [value, setValue] = useState(entry.value);
  const [isEditing, setIsEditing] = useState(false);

  const handleSave = () => {
    onSave(entry.key, value);
    setIsEditing(false);
  };

  const isSensitive = entry.key.toLowerCase().includes("password") ||
    entry.key.toLowerCase().includes("key") ||
    entry.key.toLowerCase().includes("secret");

  return (
    <div className="glass-card p-4">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <div className="font-mono text-sm text-neon-cyan mb-2">{entry.key}</div>
          {isEditing ? (
            <input
              type={isSensitive ? "password" : "text"}
              value={value}
              onChange={(e) => setValue(e.target.value)}
              className="glass-input w-full"
              autoFocus
            />
          ) : (
            <div className="text-sm text-gray-300 font-mono">
              {isSensitive && value ? "•".repeat(value.length) : value || "(not set)"}
            </div>
          )}
        </div>
        <div className="ml-4">
          {isEditing ? (
            <div className="flex gap-2">
              <Button onClick={handleSave} variant="primary" className="text-xs">
                Save
              </Button>
              <Button
                onClick={() => {
                  setValue(entry.value);
                  setIsEditing(false);
                }}
                variant="ghost"
                className="text-xs"
              >
                Cancel
              </Button>
            </div>
          ) : (
            <Button onClick={() => setIsEditing(true)} variant="secondary" className="text-xs">
              Edit
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
