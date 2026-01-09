use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use crate::storage::Database;
use crate::storage::automation::{AutomationStore, Workflow};
use crate::storage::DevOpsStore;
use crate::services::script_engine::ScriptEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowStep {
    #[serde(rename = "ExecuteScript")]
    ExecuteScript {
        script_id: i64,
        inputs: Value,
        output_var: Option<String>,
    },
    #[serde(rename = "CallCommand")]
    CallCommand {
        command: String,
        args: Value,
        output_var: Option<String>,
    },
    #[serde(rename = "Condition")]
    Condition {
        condition: String,
        if_true: Vec<WorkflowStep>,
        if_false: Vec<WorkflowStep>,
    },
    #[serde(rename = "Wait")]
    Wait {
        duration_seconds: i64,
    },
    #[serde(rename = "SendAlert")]
    SendAlert {
        message: String,
        severity: String,
        channel: Option<String>,
    },
    #[serde(rename = "SetVariable")]
    SetVariable {
        name: String,
        value: Value,
    },
    #[serde(rename = "Loop")]
    Loop {
        items: Value,
        item_var: String,
        steps: Vec<WorkflowStep>,
    },
}

#[derive(Debug, Clone)]
pub struct WorkflowContext {
    pub variables: HashMap<String, Value>,
    pub execution_id: i64,
    pub workflow_id: i64,
    pub step_index: usize,
}

pub struct WorkflowEngine {
    db: Arc<Mutex<Database>>,
    app: AppHandle,
    script_engine: Arc<ScriptEngine>,
}

impl WorkflowEngine {
    pub fn new(db: Arc<Mutex<Database>>, app: AppHandle) -> Self {
        let script_engine = Arc::new(ScriptEngine::new(db.clone(), app.clone()));
        WorkflowEngine {
            db,
            app,
            script_engine,
        }
    }

    pub async fn execute_workflow(
        &self,
        workflow_id: i64,
        trigger_data: Option<Value>,
    ) -> Result<i64> {
        // Load workflow
        let workflow = {
            let db_guard = self.db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
            let store = AutomationStore::new(db_guard.conn.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize AutomationStore: {}", e))?;
            store.list_workflows()
                .context("Failed to load workflows")?
                .into_iter()
                .find(|w| w.id == workflow_id)
                .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", workflow_id))?
        };

        if !workflow.enabled {
            return Err(anyhow::anyhow!("Workflow is disabled"));
        }

        // Parse workflow steps
        let steps: Vec<WorkflowStep> = serde_json::from_str(&workflow.steps)
            .context("Failed to parse workflow steps")?;

        // Record execution start
        let execution_id = {
            let db_guard = self.db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
            let store = AutomationStore::new(db_guard.conn.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize AutomationStore: {}", e))?;
            store.record_execution(workflow_id, "running", None)
                .context("Failed to record execution")?
        };

        // Initialize context
        let mut context = WorkflowContext {
            variables: HashMap::new(),
            execution_id,
            workflow_id,
            step_index: 0,
        };

        // Add trigger data to context
        if let Some(data) = trigger_data {
            context.variables.insert("trigger_data".to_string(), data);
        }

        // Execute steps using a work queue to avoid recursion
        let mut work_queue: Vec<WorkflowStep> = steps.into_iter().rev().collect();
        let mut result = Ok(());

        while let Some(step) = work_queue.pop() {
            context.step_index = work_queue.len();
            match self.execute_single_step(&step, &mut context).await {
                Ok(Some(mut new_steps)) => {
                    // Add new steps to work queue (in reverse order)
                    new_steps.reverse();
                    work_queue.extend(new_steps);
                }
                Ok(None) => {
                    // Step completed, continue
                }
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }

        // Update execution status
        let status = if result.is_ok() {
            "completed"
        } else {
            "failed"
        };
        let error = result.as_ref().err().map(|e| e.to_string());

        {
            let db_guard = self.db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
            let store = AutomationStore::new(db_guard.conn.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize AutomationStore: {}", e))?;
            store.record_execution(workflow_id, status, error.as_deref())
                .context("Failed to update execution status")?;
        }

        // Emit WebSocket event for real-time updates
        use tauri::Emitter;
        let _ = self.app.emit("ws-message", serde_json::json!({
            "type": "workflow-execution",
            "data": {
                "id": execution_id,
                "workflow_id": workflow_id,
                "status": status,
                "started_at": chrono::Utc::now().timestamp(),
                "completed_at": if status != "running" { Some(chrono::Utc::now().timestamp()) } else { None },
                "error": error,
            },
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }));

        result?;
        Ok(execution_id)
    }

    // Execute a single step, returning optional steps to add to work queue
    async fn execute_single_step(
        &self,
        step: &WorkflowStep,
        context: &mut WorkflowContext,
    ) -> Result<Option<Vec<WorkflowStep>>> {
        match step {
            WorkflowStep::ExecuteScript { script_id, inputs, output_var } => {
                let resolved_inputs = self.resolve_variables(inputs, context)?;
                let result = self.script_engine.execute_script(*script_id, Some(resolved_inputs)).await?;
                
                let data = if result.success {
                    result.data
                } else {
                    return Err(anyhow::anyhow!("Script execution failed: {}", result.error.unwrap_or_else(|| "Unknown error".to_string())));
                };

                if let Some(var) = output_var {
                    context.variables.insert(var.clone(), data.clone());
                }
                Ok(None)
            }
            WorkflowStep::CallCommand { command, args, output_var } => {
                let resolved_args = self.resolve_variables(args, context)?;
                
                // Invoke Tauri command via dispatcher
                let result = crate::services::CommandDispatcher::invoke_command(
                    &self.app,
                    command,
                    resolved_args,
                ).await
                .map_err(|e| anyhow::anyhow!("CallCommand failed for '{}': {}", command, e))?;
                
                if let Some(var) = output_var {
                    context.variables.insert(var.clone(), result.clone());
                }
                Ok(None)
            }
            WorkflowStep::Condition { condition, if_true, if_false } => {
                let condition_result = self.evaluate_condition(condition, context)?;
                let steps_to_execute = if condition_result { if_true } else { if_false };
                Ok(Some(steps_to_execute.clone()))
            }
            WorkflowStep::Wait { duration_seconds } => {
                tokio::time::sleep(tokio::time::Duration::from_secs(*duration_seconds as u64)).await;
                Ok(None)
            }
            WorkflowStep::SendAlert { message, severity, channel: _ } => {
                let resolved_message = self.resolve_variable_string(message, context)?;
                
                // Create DevOps alert
                let db_guard = self.db.lock()
                    .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
                let devops_store = crate::storage::DevOpsStore::new(db_guard.conn.clone());
                
                let alert_name = format!("Workflow {} - Step {}", context.workflow_id, context.step_index);
                if let Err(e) = devops_store.create_alert(&alert_name, &severity, &resolved_message, "automation") {
                    eprintln!("Failed to create alert: {}", e);
                }
                
                Ok(None)
            }
            WorkflowStep::SetVariable { name, value } => {
                let resolved_value = self.resolve_variables(value, context)?;
                context.variables.insert(name.clone(), resolved_value.clone());
                Ok(None)
            }
            WorkflowStep::Loop { items, item_var, steps } => {
                let resolved_items = self.resolve_variables(items, context)?;
                
                if let Value::Array(arr) = resolved_items {
                    // For each item, add the loop steps to the work queue
                    let mut loop_steps = Vec::new();
                    for item in arr.iter().rev() {
                        // Create a SetVariable step for the item
                        let set_var_step = WorkflowStep::SetVariable {
                            name: item_var.clone(),
                            value: item.clone(),
                        };
                        loop_steps.push(set_var_step);
                        // Add the loop body steps
                        for step in steps.iter().rev() {
                            loop_steps.push(step.clone());
                        }
                    }
                    Ok(Some(loop_steps))
                } else {
                    Err(anyhow::anyhow!("Loop items must be an array"))
                }
            }
        }
    }

    fn resolve_variables(&self, value: &Value, context: &WorkflowContext) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.starts_with("{{") && s.ends_with("}}") {
                    let var_name = s.trim_start_matches("{{").trim_end_matches("}}").trim();
                    context.variables.get(var_name)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Variable not found: {}", var_name))
                } else {
                    Ok(value.clone())
                }
            }
            Value::Object(map) => {
                let mut resolved = serde_json::Map::new();
                for (k, v) in map {
                    resolved.insert(k.clone(), self.resolve_variables(v, context)?);
                }
                Ok(Value::Object(resolved))
            }
            Value::Array(arr) => {
                let resolved: Result<Vec<Value>> = arr.iter()
                    .map(|v| self.resolve_variables(v, context))
                    .collect();
                Ok(Value::Array(resolved?))
            }
            _ => Ok(value.clone()),
        }
    }

    fn resolve_variable_string(&self, template: &str, context: &WorkflowContext) -> Result<String> {
        let mut result = template.to_string();
        for (var_name, var_value) in &context.variables {
            let placeholder = format!("{{{{{}}}}}", var_name);
            let value_str = match var_value {
                Value::String(s) => s.clone(),
                _ => serde_json::to_string(var_value).unwrap_or_else(|_| "".to_string()),
            };
            result = result.replace(&placeholder, &value_str);
        }
        Ok(result)
    }

    fn evaluate_condition(&self, condition: &str, context: &WorkflowContext) -> Result<bool> {
        let resolved = self.resolve_variable_string(condition, context)?;
        
        if resolved.contains(">") {
            let parts: Vec<&str> = resolved.split('>').collect();
            if parts.len() == 2 {
                let left: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let right: f64 = parts[1].trim().parse().unwrap_or(0.0);
                return Ok(left > right);
            }
        } else if resolved.contains("<") {
            let parts: Vec<&str> = resolved.split('<').collect();
            if parts.len() == 2 {
                let left: f64 = parts[0].trim().parse().unwrap_or(0.0);
                let right: f64 = parts[1].trim().parse().unwrap_or(0.0);
                return Ok(left < right);
            }
        } else if resolved.contains("==") {
            let parts: Vec<&str> = resolved.split("==").collect();
            if parts.len() == 2 {
                return Ok(parts[0].trim() == parts[1].trim());
            }
        }
        
        Ok(resolved.parse::<bool>().unwrap_or(false))
    }
}
