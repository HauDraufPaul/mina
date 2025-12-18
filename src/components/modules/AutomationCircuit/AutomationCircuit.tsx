import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import { Code, Play, Save, List, Workflow, Clock, CheckCircle, XCircle, Plus } from "lucide-react";

interface Script {
  id: number;
  name: string;
  content: string;
  language: string;
  created_at: number;
  updated_at: number;
  enabled: boolean;
}

interface Workflow {
  id: number;
  name: string;
  description?: string;
  trigger_type: string;
  trigger_config: string;
  steps: string;
  created_at: number;
  enabled: boolean;
}

interface WorkflowExecution {
  id: number;
  workflow_id: number;
  status: string;
  started_at: number;
  completed_at?: number;
  error?: string;
}

export default function AutomationCircuit() {
  const [scripts, setScripts] = useState<Script[]>([]);
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [executions, setExecutions] = useState<WorkflowExecution[]>([]);
  const [selectedScript, setSelectedScript] = useState<Script | null>(null);
  const [scriptContent, setScriptContent] = useState("");
  const [scriptName, setScriptName] = useState("");
  const [scriptLanguage, setScriptLanguage] = useState("javascript");
  const [view, setView] = useState<"scripts" | "workflows" | "executions">("scripts");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [scriptsData, workflowsData, executionsData] = await Promise.all([
        invoke<Script[]>("list_scripts"),
        invoke<Workflow[]>("list_workflows"),
        invoke<WorkflowExecution[]>("get_workflow_executions", {
          workflowId: null,
          limit: 20,
        }),
      ]);
      setScripts(scriptsData);
      setWorkflows(workflowsData);
      setExecutions(executionsData);
      setLoading(false);
    } catch (error) {
      console.error("Failed to load data:", error);
      setLoading(false);
    }
  };

  const handleCreateScript = async () => {
    if (!scriptName.trim()) {
      // TODO: Replace with proper error UI component
      alert("Please enter a script name");
      return;
    }

    // Validate script name (alphanumeric, underscore, hyphen)
    if (!/^[a-zA-Z0-9_-]+$/.test(scriptName.trim())) {
      alert("Script name can only contain letters, numbers, underscores, and hyphens");
      return;
    }

    try {
      await invoke("create_script", {
        name: scriptName.trim(),
        content: scriptContent || "// Your code here",
        language: scriptLanguage,
      });
      setScriptName("");
      setScriptContent("");
      await loadData();
    } catch (error) {
      // TODO: Replace with proper error UI component
      console.error("Failed to create script:", error);
      alert(`Failed to create script: ${error}`);
    }
  };

  const handleSelectScript = async (id: number) => {
    try {
      const script = await invoke<Script | null>("get_script", { id });
      if (script) {
        setSelectedScript(script);
        setScriptContent(script.content);
        setScriptName(script.name);
        setScriptLanguage(script.language);
      }
    } catch (error) {
      console.error("Failed to load script:", error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case "completed":
        return <CheckCircle className="w-4 h-4 text-neon-green" />;
      case "failed":
        return <XCircle className="w-4 h-4 text-neon-red" />;
      case "running":
        return <Clock className="w-4 h-4 text-neon-amber animate-spin" />;
      default:
        return <Clock className="w-4 h-4 text-gray-400" />;
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  if (loading) {
    return <div className="text-center">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2 phosphor-glow-cyan">
            Automation Circuit
          </h1>
          <p className="text-gray-400">Script and workflow automation</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant={view === "scripts" ? "primary" : "secondary"}
            onClick={() => setView("scripts")}
          >
            <Code className="w-4 h-4 mr-2" />
            Scripts
          </Button>
          <Button
            variant={view === "workflows" ? "primary" : "secondary"}
            onClick={() => setView("workflows")}
          >
            <Workflow className="w-4 h-4 mr-2" />
            Workflows
          </Button>
          <Button
            variant={view === "executions" ? "primary" : "secondary"}
            onClick={() => setView("executions")}
          >
            <List className="w-4 h-4 mr-2" />
            Executions
          </Button>
        </div>
      </div>

      {view === "scripts" && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-1">
            <Card title="Scripts">
              <div className="space-y-2">
                <Button variant="primary" className="w-full" onClick={handleCreateScript}>
                  <Plus className="w-4 h-4 mr-2" />
                  New Script
                </Button>
                <div className="space-y-1 max-h-96 overflow-y-auto">
                  {scripts.map((script) => (
                    <button
                      key={script.id}
                      onClick={() => handleSelectScript(script.id)}
                      className={`w-full text-left p-2 rounded glass-card transition-all ${
                        selectedScript?.id === script.id
                          ? "border-2 border-neon-cyan"
                          : "hover:border border-white/10"
                      }`}
                    >
                      <div className="font-semibold text-sm">{script.name}</div>
                      <div className="text-xs text-gray-400">{script.language}</div>
                    </button>
                  ))}
                </div>
              </div>
            </Card>
          </div>

          <div className="lg:col-span-2">
            <Card title={selectedScript ? `Edit: ${selectedScript.name}` : "New Script"}>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Name</label>
                  <input
                    type="text"
                    value={scriptName}
                    onChange={(e) => setScriptName(e.target.value)}
                    className="glass-input w-full"
                    placeholder="script_name"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Language</label>
                  <select
                    value={scriptLanguage}
                    onChange={(e) => setScriptLanguage(e.target.value)}
                    className="glass-input w-full"
                  >
                    <option value="javascript">JavaScript</option>
                    <option value="typescript">TypeScript</option>
                    <option value="python">Python</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-2">Code</label>
                  <textarea
                    value={scriptContent}
                    onChange={(e) => setScriptContent(e.target.value)}
                    className="glass-input w-full font-mono text-sm"
                    rows={20}
                    placeholder="// Your code here"
                  />
                </div>
                <div className="flex gap-2">
                  <Button variant="primary" onClick={handleCreateScript}>
                    <Save className="w-4 h-4 mr-2" />
                    Save
                  </Button>
                  <Button variant="secondary">
                    <Play className="w-4 h-4 mr-2" />
                    Run
                  </Button>
                </div>
              </div>
            </Card>
          </div>
        </div>
      )}

      {view === "workflows" && (
        <Card title="Workflows">
          <div className="space-y-4">
            <Button variant="primary">
              <Plus className="w-4 h-4 mr-2" />
              Create Workflow
            </Button>
            {workflows.length === 0 ? (
              <div className="text-center text-gray-400 py-8">
                <Workflow className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>No workflows created yet</p>
              </div>
            ) : (
              workflows.map((workflow) => (
                <div key={workflow.id} className="glass-card p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-semibold">{workflow.name}</div>
                      {workflow.description && (
                        <div className="text-sm text-gray-400">{workflow.description}</div>
                      )}
                      <div className="text-xs text-gray-500 mt-1">
                        Trigger: {workflow.trigger_type}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span
                        className={`text-xs px-2 py-1 rounded ${
                          workflow.enabled
                            ? "bg-neon-green/20 text-neon-green"
                            : "bg-gray-500/20 text-gray-500"
                        }`}
                      >
                        {workflow.enabled ? "Enabled" : "Disabled"}
                      </span>
                      <Button variant="secondary">
                        <Play className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </Card>
      )}

      {view === "executions" && (
        <Card title="Execution History">
          <div className="space-y-3">
            {executions.length === 0 ? (
              <div className="text-center text-gray-400 py-8">
                <List className="w-12 h-12 mx-auto mb-4 text-gray-500" />
                <p>No executions yet</p>
              </div>
            ) : (
              executions.map((exec) => (
                <div key={exec.id} className="glass-card p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      {getStatusIcon(exec.status)}
                      <div>
                        <div className="font-semibold">Workflow #{exec.workflow_id}</div>
                        <div className="text-xs text-gray-400">
                          Started: {formatTime(exec.started_at)}
                        </div>
                        {exec.completed_at && (
                          <div className="text-xs text-gray-400">
                            Duration:{" "}
                            {(exec.completed_at - exec.started_at).toFixed(2)}s
                          </div>
                        )}
                        {exec.error && (
                          <div className="text-xs text-neon-red mt-1">{exec.error}</div>
                        )}
                      </div>
                    </div>
                    <span
                      className={`text-xs px-2 py-1 rounded ${
                        exec.status === "completed"
                          ? "bg-neon-green/20 text-neon-green"
                          : exec.status === "failed"
                          ? "bg-neon-red/20 text-neon-red"
                          : "bg-neon-amber/20 text-neon-amber"
                      }`}
                    >
                      {exec.status.toUpperCase()}
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>
        </Card>
      )}
    </div>
  );
}
