use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use cron::Schedule;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::interval;

use crate::storage::Database;
use crate::storage::automation::{AutomationStore, Workflow};
use crate::services::workflow_engine::WorkflowEngine;

struct ScheduledWorkflow {
    workflow_id: i64,
    cron_expr: Schedule,
    next_execution: DateTime<Utc>,
    timezone: Option<String>,
}

pub struct WorkflowScheduler {
    schedules: Arc<Mutex<HashMap<i64, ScheduledWorkflow>>>,
    db: Arc<Mutex<Database>>,
    workflow_engine: Arc<WorkflowEngine>,
    app: AppHandle,
    stop_signal: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl WorkflowScheduler {
    pub fn new(
        db: Arc<Mutex<Database>>,
        workflow_engine: Arc<WorkflowEngine>,
        app: AppHandle,
    ) -> Self {
        WorkflowScheduler {
            schedules: Arc::new(Mutex::new(HashMap::new())),
            db,
            workflow_engine,
            app,
            stop_signal: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self) {
        let schedules = self.schedules.clone();
        let db = self.db.clone();
        let workflow_engine = self.workflow_engine.clone();
        let app = self.app.clone();
        let stop_signal = self.stop_signal.clone();

        // Reload schedules from database
        self.reload_schedules();

        // Create stop channel
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        {
            let mut stop_guard = stop_signal.lock().unwrap();
            *stop_guard = Some(tx);
        }

        // Start scheduler loop
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Continue with normal scheduling logic
                    }
                    _ = &mut rx => {
                        eprintln!("Workflow scheduler stopped");
                        break;
                    }
                }

                // Check for workflows that need to run
                let now = Utc::now();
                let workflows_to_run: Vec<i64> = {
                    let schedules_guard = schedules.lock().unwrap();
                    schedules_guard
                        .values()
                        .filter(|s| s.next_execution <= now)
                        .map(|s| s.workflow_id)
                        .collect()
                };

                // Execute workflows
                for workflow_id in workflows_to_run {
                    let engine = workflow_engine.clone();
                    let app_clone = app.clone();
                    
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = engine.execute_workflow(workflow_id, None).await {
                            eprintln!("Failed to execute scheduled workflow {}: {}", workflow_id, e);
                        }
                    });

                    // Update next execution time
                    if let Ok(mut schedules_guard) = schedules.lock() {
                        if let Some(scheduled) = schedules_guard.get_mut(&workflow_id) {
                            // Calculate next execution
                            scheduled.next_execution = scheduled.cron_expr
                                .upcoming(Utc)
                                .next()
                                .unwrap_or(now + chrono::Duration::hours(24));
                        }
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        let mut stop_guard = self.stop_signal.lock().unwrap();
        if let Some(tx) = stop_guard.take() {
            let _ = tx.send(());
        }
    }

    pub fn restart(&self) {
        self.stop();
        // Clear stop signal for new start
        {
            let mut stop_guard = self.stop_signal.lock().unwrap();
            *stop_guard = None;
        }
        self.start();
    }

    pub fn reload_schedules(&self) {
        let db_guard = self.db.lock().unwrap();
        let store = AutomationStore::new(db_guard.conn.clone()).unwrap();
        
        let workflows = store.list_workflows().unwrap_or_default();
        let mut schedules_guard = self.schedules.lock().unwrap();
        schedules_guard.clear();

        let now = Utc::now();

        for workflow in workflows {
            if !workflow.enabled || workflow.trigger_type != "schedule" {
                continue;
            }

            // Parse trigger config
            let trigger_config: Value = serde_json::from_str(&workflow.trigger_config)
                .unwrap_or(json!({}));

            if let Some(cron_str) = trigger_config.get("cron").and_then(|v| v.as_str()) {
                if let Ok(schedule) = Schedule::from_str(cron_str) {
                    let next_execution = schedule
                        .upcoming(Utc)
                        .next()
                        .unwrap_or(now + chrono::Duration::hours(24));

                    schedules_guard.insert(workflow.id, ScheduledWorkflow {
                        workflow_id: workflow.id,
                        cron_expr: schedule,
                        next_execution,
                        timezone: trigger_config.get("timezone")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }
    }

    pub fn get_next_execution(&self, workflow_id: i64) -> Option<DateTime<Utc>> {
        let schedules_guard = self.schedules.lock().ok()?;
        schedules_guard.get(&workflow_id).map(|s| s.next_execution)
    }
}

