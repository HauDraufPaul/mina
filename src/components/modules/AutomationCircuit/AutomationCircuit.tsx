import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../../ui/Card";
import Button from "../../ui/Button";
import Tabs from "../../ui/Tabs";
import Modal from "../../ui/Modal";
import { useErrorHandler, validateInput } from "@/utils/errorHandler";
import { Code, Play, Save, List, Workflow, Clock, CheckCircle, XCircle, Plus, Edit } from "lucide-react";
import WorkflowEditor from "./WorkflowEditor";
import { realtimeService } from "@/services/realtimeService";

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
  const errorHandler = useErrorHandler();
  const [scripts, setScripts] = useState<Script[]>([]);
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [executions, setExecutions] = useState<WorkflowExecution[]>([]);
  const [selectedScript, setSelectedScript] = useState<Script | null>(null);
  const [scriptContent, setScriptContent] = useState("");
  const [scriptName, setScriptName] = useState("");
  const [scriptLanguage, setScriptLanguage] = useState("javascript");
  const [view, setView] = useState<"scripts" | "workflows" | "executions">("scripts");
  const [loading, setLoading] = useState(true);
  const [showWorkflowModal, setShowWorkflowModal] = useState(false);
  const [workflowName, setWorkflowName] = useState("");
  const [workflowDescription, setWorkflowDescription] = useState("");
  const [workflowTriggerType, setWorkflowTriggerType] = useState("manual");
  const [workflowTriggerConfig, setWorkflowTriggerConfig] = useState("{}");
  const [executionOutput, setExecutionOutput] = useState<{
    success: boolean;
    data: any;
    stdout: string;
    stderr: string;
    executionTimeMs: number;
    error?: string;
  } | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [selectedWorkflow, setSelectedWorkflow] = useState<Workflow | null>(null);
  const [showWorkflowEditor, setShowWorkflowEditor] = useState(false);

  useEffect(() => {
    loadData();
    
    // Set up real-time execution updates via WebSocket
    const unsubscribe = realtimeService.subscribe("workflow-execution", (data: unknown) => {
      const execution = data as WorkflowExecution;
      setExecutions((prev) => {
        const existing = prev.findIndex((e) => e.id === execution.id);
        if (existing >= 0) {
          const updated = [...prev];
          updated[existing] = execution;
          return updated;
        } else {
          return [execution, ...prev].slice(0, 20);
        }
      });
    });

    return () => {
      unsubscribe();
    };
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
      errorHandler.showError("Failed to load data", error);
      setLoading(false);
    }
  };

  const handleSaveScript = async () => {
    const trimmedName = scriptName.trim();
    
    if (!validateInput(trimmedName, { required: true }, errorHandler)) {
      return;
    }

    if (!validateInput(trimmedName, {
      pattern: /^[a-zA-Z0-9_-]+$/,
      patternMessage: "Script name can only contain letters, numbers, underscores, and hyphens",
    }, errorHandler)) {
      return;
    }

    try {
      if (selectedScript) {
        // Update existing script
        await invoke("update_script", {
          id: selectedScript.id,
          name: trimmedName,
          content: scriptContent || "// Your code here",
          language: scriptLanguage,
        });
        errorHandler.showSuccess("Script updated successfully");
      } else {
        // Create new script
        await invoke("create_script", {
          name: trimmedName,
          content: scriptContent || "// Your code here",
          language: scriptLanguage,
        });
        errorHandler.showSuccess("Script created successfully");
        setScriptName("");
        setScriptContent("");
      }
      await loadData();
    } catch (error) {
      errorHandler.showError(selectedScript ? "Failed to update script" : "Failed to create script", error);
    }
  };

  const handleDeleteScript = async (id: number) => {
    if (!confirm("Are you sure you want to delete this script?")) {
      return;
    }

    try {
      await invoke("delete_script", { id });
      errorHandler.showSuccess("Script deleted successfully");
      if (selectedScript?.id === id) {
        setSelectedScript(null);
        setScriptName("");
        setScriptContent("");
      }
      await loadData();
    } catch (error) {
      errorHandler.showError("Failed to delete script", error);
    }
  };

  const handleRunScript = async () => {
    if (!selectedScript) {
      errorHandler.showError("Please select a script to run", new Error("No script selected"));
      return;
    }

    setIsExecuting(true);
    setExecutionOutput(null);

    try {
      const result = await invoke<{
        success: boolean;
        data: any;
        stdout: string;
        stderr: string;
        execution_time_ms: number;
        error?: string;
      }>("execute_script", {
        scriptId: selectedScript.id,
        inputs: null,
      });

      setExecutionOutput({
        success: result.success,
        data: result.data,
        stdout: result.stdout,
        stderr: result.stderr,
        executionTimeMs: result.execution_time_ms,
        error: result.error,
      });

      if (result.success) {
        errorHandler.showSuccess(`Script executed successfully in ${result.execution_time_ms}ms`);
      } else {
        errorHandler.showError("Script execution failed", new Error(result.error || "Unknown error"));
      }
    } catch (error) {
      errorHandler.showError("Failed to run script", error);
      setExecutionOutput({
        success: false,
        data: null,
        stdout: "",
        stderr: "",
        executionTimeMs: 0,
        error: String(error),
      });
    } finally {
      setIsExecuting(false);
    }
  };

  const handleCreateWorkflow = async () => {
    const trimmedName = workflowName.trim();
    
    if (!validateInput(trimmedName, { required: true }, errorHandler)) {
      return;
    }

    try {
      await invoke("create_workflow", {
        name: trimmedName,
        description: workflowDescription.trim() || null,
        triggerType: workflowTriggerType,
        triggerConfig: workflowTriggerConfig,
        steps: "[]", // Empty steps array for now
      });
      errorHandler.showSuccess("Workflow created successfully");
      setShowWorkflowModal(false);
      setWorkflowName("");
      setWorkflowDescription("");
      setWorkflowTriggerType("manual");
      setWorkflowTriggerConfig("{}");
      await loadData();
    } catch (error) {
      errorHandler.showError("Failed to create workflow", error);
    }
  };

  const handleExecuteWorkflow = async (workflowId: number) => {
    try {
      const executionId = await invoke<number>("execute_workflow", {
        workflowId,
        triggerData: null,
      });
      
      errorHandler.showSuccess(`Workflow execution started (ID: ${executionId})`);
      await loadData();
    } catch (error) {
      errorHandler.showError("Failed to execute workflow", error);
    }
  };

  const handleEditWorkflow = async (workflowId: number) => {
    try {
      const workflow = await invoke<Workflow | null>("get_workflow", { id: workflowId });
      if (workflow) {
        setSelectedWorkflow(workflow);
        setShowWorkflowEditor(true);
      }
    } catch (error) {
      errorHandler.showError("Failed to load workflow", error);
    }
  };

  const handleSaveWorkflow = async (steps: any[]) => {
    if (!selectedWorkflow) return;

    try {
      await invoke("update_workflow", {
        id: selectedWorkflow.id,
        steps: JSON.stringify(steps),
      });
      errorHandler.showSuccess("Workflow updated successfully");
      setShowWorkflowEditor(false);
      setSelectedWorkflow(null);
      await loadData();
    } catch (error) {
      errorHandler.showError("Failed to save workflow", error);
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
      errorHandler.showError("Failed to load script", error);
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
        <Tabs
          items={[
            {
              id: "scripts",
              label: "Scripts",
              icon: <Code className="w-4 h-4" />,
            },
            {
              id: "workflows",
              label: "Workflows",
              icon: <Workflow className="w-4 h-4" />,
            },
            {
              id: "executions",
              label: "Executions",
              icon: <List className="w-4 h-4" />,
            },
          ]}
          activeTab={view}
          onTabChange={(tabId) => setView(tabId as "scripts" | "workflows" | "executions")}
        />
      </div>

      {view === "scripts" && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-1">
            <Card title="Scripts">
              <div className="space-y-2">
                <Button 
                  variant="primary" 
                  className="w-full" 
                  onClick={() => {
                    setSelectedScript(null);
                    setScriptName("");
                    setScriptContent("");
                    setScriptLanguage("javascript");
                    setExecutionOutput(null);
                  }}
                >
                  <Plus className="w-4 h-4 mr-2" />
                  New Script
                </Button>
                <div className="space-y-1 max-h-96 overflow-y-auto">
                  {scripts.map((script) => (
                    <div
                      key={script.id}
                      className={`w-full p-2 rounded glass-card transition-all ${
                        selectedScript?.id === script.id
                          ? "border-2 border-neon-cyan"
                          : "hover:border border-white/10"
                      }`}
                    >
                      <button
                        onClick={() => handleSelectScript(script.id)}
                        className="w-full text-left"
                      >
                        <div className="font-semibold text-sm">{script.name}</div>
                        <div className="text-xs text-gray-400">{script.language}</div>
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteScript(script.id);
                        }}
                        className="mt-1 text-xs text-neon-red hover:text-neon-red/80"
                      >
                        Delete
                      </button>
                    </div>
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
                  <Button variant="primary" onClick={handleSaveScript}>
                    <Save className="w-4 h-4 mr-2" />
                    {selectedScript ? "Update" : "Create"}
                  </Button>
                  {selectedScript && (
                    <Button 
                      variant="secondary" 
                      onClick={handleRunScript}
                      disabled={isExecuting}
                    >
                      <Play className="w-4 h-4 mr-2" />
                      {isExecuting ? "Running..." : "Run"}
                    </Button>
                  )}
                  {selectedScript && (
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setSelectedScript(null);
                        setScriptName("");
                        setScriptContent("");
                        setScriptLanguage("javascript");
                        setExecutionOutput(null);
                      }}
                    >
                      New Script
                    </Button>
                  )}
                </div>
                
                {executionOutput && (
                  <div className="mt-4 border-t border-white/10 pt-4">
                    <div className="flex items-center justify-between mb-2">
                      <label className="block text-sm text-gray-400">Execution Output</label>
                      <span className={`text-xs px-2 py-1 rounded ${
                        executionOutput.success
                          ? "bg-neon-green/20 text-neon-green"
                          : "bg-neon-red/20 text-neon-red"
                      }`}>
                        {executionOutput.success ? "Success" : "Failed"} ({executionOutput.executionTimeMs}ms)
                      </span>
                    </div>
                    {executionOutput.stdout && (
                      <div className="mb-2">
                        <div className="text-xs text-gray-400 mb-1">STDOUT:</div>
                        <pre className="glass-card p-3 text-xs font-mono overflow-x-auto max-h-32 overflow-y-auto">
                          {executionOutput.stdout}
                        </pre>
                      </div>
                    )}
                    {executionOutput.stderr && (
                      <div className="mb-2">
                        <div className="text-xs text-neon-red mb-1">STDERR:</div>
                        <pre className="glass-card p-3 text-xs font-mono text-neon-red overflow-x-auto max-h-32 overflow-y-auto">
                          {executionOutput.stderr}
                        </pre>
                      </div>
                    )}
                    {executionOutput.data && (
                      <div className="mb-2">
                        <div className="text-xs text-gray-400 mb-1">Result:</div>
                        <pre className="glass-card p-3 text-xs font-mono overflow-x-auto max-h-32 overflow-y-auto">
                          {JSON.stringify(executionOutput.data, null, 2)}
                        </pre>
                      </div>
                    )}
                    {executionOutput.error && (
                      <div className="text-xs text-neon-red">
                        Error: {executionOutput.error}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </Card>
          </div>
        </div>
      )}

      {view === "workflows" && (
        <Card title="Workflows">
          <div className="space-y-4">
            <Button variant="primary" onClick={() => setShowWorkflowModal(true)}>
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
                      <Button variant="secondary" onClick={() => handleEditWorkflow(workflow.id)}>
                        <Edit className="w-4 h-4" />
                      </Button>
                      <Button variant="secondary" onClick={() => handleExecuteWorkflow(workflow.id)}>
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

      {showWorkflowEditor && selectedWorkflow && (
        <Modal
          isOpen={showWorkflowEditor}
          onClose={() => {
            setShowWorkflowEditor(false);
            setSelectedWorkflow(null);
          }}
          title="Edit Workflow"
        >
          <WorkflowEditor
            workflowId={selectedWorkflow.id}
            workflowName={selectedWorkflow.name}
            steps={selectedWorkflow.steps}
            onSave={handleSaveWorkflow}
            onClose={() => {
              setShowWorkflowEditor(false);
              setSelectedWorkflow(null);
            }}
            scripts={scripts.map((s) => ({ id: s.id, name: s.name }))}
          />
        </Modal>
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

      {/* Create Workflow Modal */}
      <Modal
        isOpen={showWorkflowModal}
        onClose={() => {
          setShowWorkflowModal(false);
          setWorkflowName("");
          setWorkflowDescription("");
          setWorkflowTriggerType("manual");
          setWorkflowTriggerConfig("{}");
        }}
        title="Create Workflow"
      >
        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Workflow Name
            </label>
            <input
              type="text"
              value={workflowName}
              onChange={(e) => setWorkflowName(e.target.value)}
              className="glass-input w-full"
              placeholder="e.g., Daily Data Sync, Market Alert..."
              autoFocus
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Description (Optional)
            </label>
            <textarea
              value={workflowDescription}
              onChange={(e) => setWorkflowDescription(e.target.value)}
              className="glass-input w-full"
              rows={3}
              placeholder="Describe what this workflow does..."
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Trigger Type
            </label>
            <select
              value={workflowTriggerType}
              onChange={(e) => setWorkflowTriggerType(e.target.value)}
              className="glass-input w-full"
            >
              <option value="manual">Manual</option>
              <option value="schedule">Schedule</option>
              <option value="event">Event</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Trigger Config (JSON)
            </label>
            <textarea
              value={workflowTriggerConfig}
              onChange={(e) => setWorkflowTriggerConfig(e.target.value)}
              className="glass-input w-full font-mono text-sm"
              rows={4}
              placeholder='{"cron": "0 0 * * *"} or {"event": "market_open"}'
            />
          </div>
          <div className="flex gap-2 justify-end">
            <Button
              variant="secondary"
              onClick={() => {
                setShowWorkflowModal(false);
                setWorkflowName("");
                setWorkflowDescription("");
                setWorkflowTriggerType("manual");
                setWorkflowTriggerConfig("{}");
              }}
            >
              Cancel
            </Button>
            <Button variant="primary" onClick={handleCreateWorkflow}>
              <Save className="w-4 h-4 mr-2" />
              Create
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
