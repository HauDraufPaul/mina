import { useState, useEffect } from "react";
import Button from "../../ui/Button";
import { Trash2, ChevronDown, ChevronRight, Code, Play, AlertCircle, Clock, Settings, Repeat } from "lucide-react";

interface WorkflowStep {
  type: "ExecuteScript" | "CallCommand" | "Condition" | "Wait" | "SendAlert" | "SetVariable" | "Loop";
  [key: string]: any;
}

interface WorkflowEditorProps {
  workflowId?: number;
  workflowName: string;
  steps: string;
  onSave: (steps: WorkflowStep[]) => Promise<void>;
  onClose: () => void;
  scripts: Array<{ id: number; name: string }>;
}

export default function WorkflowEditor({
  workflowName,
  steps,
  onSave,
  onClose,
  scripts,
}: WorkflowEditorProps) {
  const [workflowSteps, setWorkflowSteps] = useState<WorkflowStep[]>([]);
  const [expandedSteps, setExpandedSteps] = useState<Set<number>>(new Set());
  const [editingStep, setEditingStep] = useState<number | null>(null);

  useEffect(() => {
    try {
      const parsed = JSON.parse(steps || "[]");
      setWorkflowSteps(Array.isArray(parsed) ? parsed : []);
    } catch {
      setWorkflowSteps([]);
    }
  }, [steps]);

  const toggleStep = (index: number) => {
    const newExpanded = new Set(expandedSteps);
    if (newExpanded.has(index)) {
      newExpanded.delete(index);
    } else {
      newExpanded.add(index);
    }
    setExpandedSteps(newExpanded);
  };

  const addStep = (type: WorkflowStep["type"]) => {
    const newStep: WorkflowStep = { type };
    
    switch (type) {
      case "ExecuteScript":
        newStep.script_id = scripts[0]?.id || 0;
        newStep.inputs = {};
        newStep.output_var = null;
        break;
      case "CallCommand":
        newStep.command = "";
        newStep.args = {};
        newStep.output_var = null;
        break;
      case "Condition":
        newStep.condition = "";
        newStep.if_true = [];
        newStep.if_false = [];
        break;
      case "Wait":
        newStep.duration_seconds = 1;
        break;
      case "SendAlert":
        newStep.message = "";
        newStep.severity = "info";
        newStep.channel = null;
        break;
      case "SetVariable":
        newStep.name = "";
        newStep.value = null;
        break;
      case "Loop":
        newStep.items = [];
        newStep.item_var = "item";
        newStep.steps = [];
        break;
    }
    
    setWorkflowSteps([...workflowSteps, newStep]);
  };

  const removeStep = (index: number) => {
    setWorkflowSteps(workflowSteps.filter((_, i) => i !== index));
  };

  const updateStep = (index: number, updates: Partial<WorkflowStep>) => {
    const newSteps = [...workflowSteps];
    newSteps[index] = { ...newSteps[index], ...updates };
    setWorkflowSteps(newSteps);
  };

  const getStepIcon = (type: WorkflowStep["type"]) => {
    switch (type) {
      case "ExecuteScript":
        return <Code className="w-4 h-4" />;
      case "CallCommand":
        return <Play className="w-4 h-4" />;
      case "Condition":
        return <ChevronRight className="w-4 h-4" />;
      case "Wait":
        return <Clock className="w-4 h-4" />;
      case "SendAlert":
        return <AlertCircle className="w-4 h-4" />;
      case "SetVariable":
        return <Settings className="w-4 h-4" />;
      case "Loop":
        return <Repeat className="w-4 h-4" />;
      default:
        return null;
    }
  };

  const renderStepEditor = (step: WorkflowStep, index: number) => {
    const isExpanded = expandedSteps.has(index);
    const isEditing = editingStep === index;

    return (
      <div key={index} className="glass-card p-3 mb-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 flex-1">
            <button
              onClick={() => toggleStep(index)}
              className="text-gray-400 hover:text-white"
            >
              {isExpanded ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
            </button>
            {getStepIcon(step.type)}
            <span className="font-semibold text-sm">{step.type}</span>
            {step.type === "ExecuteScript" && step.script_id && (
              <span className="text-xs text-gray-400">
                Script: {scripts.find((s) => s.id === step.script_id)?.name || step.script_id}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="secondary"
              onClick={() => setEditingStep(isEditing ? null : index)}
            >
              {isEditing ? "Done" : "Edit"}
            </Button>
            <Button variant="secondary" onClick={() => removeStep(index)}>
              <Trash2 className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {isExpanded && (
          <div className="mt-3 space-y-2">
            {step.type === "ExecuteScript" && (
              <div className="space-y-2">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Script</label>
                  <select
                    value={step.script_id || ""}
                    onChange={(e) => updateStep(index, { script_id: parseInt(e.target.value) })}
                    className="glass-input w-full text-sm"
                  >
                    <option value="">Select script...</option>
                    {scripts.map((s) => (
                      <option key={s.id} value={s.id}>
                        {s.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Inputs (JSON)</label>
                  <textarea
                    value={JSON.stringify(step.inputs || {}, null, 2)}
                    onChange={(e) => {
                      try {
                        updateStep(index, { inputs: JSON.parse(e.target.value) });
                      } catch {}
                    }}
                    className="glass-input w-full font-mono text-xs"
                    rows={3}
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Output Variable (optional)</label>
                  <input
                    type="text"
                    value={step.output_var || ""}
                    onChange={(e) => updateStep(index, { output_var: e.target.value || null })}
                    className="glass-input w-full text-sm"
                    placeholder="result"
                  />
                </div>
              </div>
            )}

            {step.type === "CallCommand" && (
              <div className="space-y-2">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Command</label>
                  <input
                    type="text"
                    value={step.command || ""}
                    onChange={(e) => updateStep(index, { command: e.target.value })}
                    className="glass-input w-full text-sm"
                    placeholder="get_system_metrics"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Arguments (JSON)</label>
                  <textarea
                    value={JSON.stringify(step.args || {}, null, 2)}
                    onChange={(e) => {
                      try {
                        updateStep(index, { args: JSON.parse(e.target.value) });
                      } catch {}
                    }}
                    className="glass-input w-full font-mono text-xs"
                    rows={3}
                  />
                </div>
              </div>
            )}

            {step.type === "Wait" && (
              <div>
                <label className="block text-xs text-gray-400 mb-1">Duration (seconds)</label>
                <input
                  type="number"
                  value={step.duration_seconds || 1}
                  onChange={(e) => updateStep(index, { duration_seconds: parseInt(e.target.value) || 1 })}
                  className="glass-input w-full text-sm"
                  min="1"
                />
              </div>
            )}

            {step.type === "SendAlert" && (
              <div className="space-y-2">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Message</label>
                  <input
                    type="text"
                    value={step.message || ""}
                    onChange={(e) => updateStep(index, { message: e.target.value })}
                    className="glass-input w-full text-sm"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Severity</label>
                  <select
                    value={step.severity || "info"}
                    onChange={(e) => updateStep(index, { severity: e.target.value })}
                    className="glass-input w-full text-sm"
                  >
                    <option value="info">Info</option>
                    <option value="warning">Warning</option>
                    <option value="critical">Critical</option>
                  </select>
                </div>
              </div>
            )}

            {step.type === "SetVariable" && (
              <div className="space-y-2">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Variable Name</label>
                  <input
                    type="text"
                    value={step.name || ""}
                    onChange={(e) => updateStep(index, { name: e.target.value })}
                    className="glass-input w-full text-sm"
                    placeholder="myVar"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Value (JSON)</label>
                  <textarea
                    value={JSON.stringify(step.value !== null ? step.value : "", null, 2)}
                    onChange={(e) => {
                      try {
                        updateStep(index, { value: JSON.parse(e.target.value) });
                      } catch {}
                    }}
                    className="glass-input w-full font-mono text-xs"
                    rows={2}
                  />
                </div>
              </div>
            )}

            {step.type === "Condition" && (
              <div className="space-y-2">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Condition</label>
                  <input
                    type="text"
                    value={step.condition || ""}
                    onChange={(e) => updateStep(index, { condition: e.target.value })}
                    className="glass-input w-full text-sm"
                    placeholder="price > 100"
                  />
                </div>
                <div className="text-xs text-gray-500">
                  If true: {step.if_true?.length || 0} steps | If false: {step.if_false?.length || 0} steps
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Edit Workflow: {workflowName}</h3>
        <Button variant="secondary" onClick={onClose}>
          Close
        </Button>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="text-sm font-semibold">Steps</label>
          <div className="flex gap-2">
            <select
              onChange={(e) => {
                if (e.target.value) {
                  addStep(e.target.value as WorkflowStep["type"]);
                  e.target.value = "";
                }
              }}
              className="glass-input text-sm"
            >
              <option value="">Add Step...</option>
              <option value="ExecuteScript">Execute Script</option>
              <option value="CallCommand">Call Command</option>
              <option value="Condition">Condition</option>
              <option value="Wait">Wait</option>
              <option value="SendAlert">Send Alert</option>
              <option value="SetVariable">Set Variable</option>
              <option value="Loop">Loop</option>
            </select>
          </div>
        </div>

        {workflowSteps.length === 0 ? (
          <div className="text-center text-gray-400 py-8">
            <p>No steps yet. Add a step to get started.</p>
          </div>
        ) : (
          workflowSteps.map((step, index) => renderStepEditor(step, index))
        )}
      </div>

      <div className="flex gap-2 justify-end">
        <Button variant="secondary" onClick={onClose}>
          Cancel
        </Button>
        <Button
          variant="primary"
          onClick={() => onSave(workflowSteps)}
        >
          Save Workflow
        </Button>
      </div>
    </div>
  );
}

