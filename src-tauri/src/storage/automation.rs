use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: i64,
    pub name: String,
    pub content: String,
    pub language: String, // "javascript", "typescript", "python"
    pub created_at: i64,
    pub updated_at: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String, // "schedule", "event", "manual"
    pub trigger_config: String, // JSON
    pub steps: String, // JSON array of step IDs
    pub created_at: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: i64,
    pub workflow_id: i64,
    pub status: String, // "running", "completed", "failed"
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error: Option<String>,
}

pub struct AutomationStore {
    conn: Arc<Mutex<Connection>>,
}

impl AutomationStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self> {
        let store = AutomationStore { conn };
        store.init_schema()
            .map_err(|e| anyhow::anyhow!("Failed to initialize Automation store schema: {}", e))?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS scripts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL,
                language TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS workflows (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                trigger_type TEXT NOT NULL,
                trigger_config TEXT NOT NULL,
                steps TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS workflow_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                completed_at INTEGER,
                error TEXT,
                FOREIGN KEY (workflow_id) REFERENCES workflows(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_executions_workflow ON workflow_executions(workflow_id)",
            [],
        )?;

        Ok(())
    }

    pub fn create_script(&self, name: &str, content: &str, language: &str) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO scripts (name, content, language, created_at, updated_at, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![name, content, language, now, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_scripts(&self) -> Result<Vec<Script>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, content, language, created_at, updated_at, enabled FROM scripts ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Script {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
                language: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                enabled: row.get::<_, i64>(6)? == 1,
            })
        })?;

        let mut scripts = Vec::new();
        for row in rows {
            scripts.push(row?);
        }
        Ok(scripts)
    }

    pub fn get_script(&self, id: i64) -> Result<Option<Script>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        let script: Option<Script> = conn
            .query_row(
                "SELECT id, name, content, language, created_at, updated_at, enabled FROM scripts WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Script {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        content: row.get(2)?,
                        language: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                        enabled: row.get::<_, i64>(6)? == 1,
                    })
                },
            )
            .optional()?;

        Ok(script)
    }

    pub fn create_workflow(
        &self,
        name: &str,
        description: Option<&str>,
        trigger_type: &str,
        trigger_config: &str,
        steps: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO workflows (name, description, trigger_type, trigger_config, steps, created_at, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            params![name, description, trigger_type, trigger_config, steps, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_workflows(&self) -> Result<Vec<Workflow>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, trigger_type, trigger_config, steps, created_at, enabled
             FROM workflows ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Workflow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                trigger_type: row.get(3)?,
                trigger_config: row.get(4)?,
                steps: row.get(5)?,
                created_at: row.get(6)?,
                enabled: row.get::<_, i64>(7)? == 1,
            })
        })?;

        let mut workflows = Vec::new();
        for row in rows {
            workflows.push(row?);
        }
        Ok(workflows)
    }

    pub fn record_execution(
        &self,
        workflow_id: i64,
        status: &str,
        error: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        let completed_at = if status == "completed" || status == "failed" {
            Some(now)
        } else {
            None
        };

        conn.execute(
            "INSERT INTO workflow_executions (workflow_id, status, started_at, completed_at, error)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![workflow_id, status, now, completed_at, error],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_executions(&self, workflow_id: Option<i64>, limit: i32) -> Result<Vec<WorkflowExecution>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut executions = Vec::new();
        
        if let Some(wf_id) = workflow_id {
            let mut stmt = conn.prepare(
                "SELECT id, workflow_id, status, started_at, completed_at, error
                 FROM workflow_executions
                 WHERE workflow_id = ?1
                 ORDER BY started_at DESC
                 LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![wf_id, limit], |row| {
                Ok(WorkflowExecution {
                    id: row.get(0)?,
                    workflow_id: row.get(1)?,
                    status: row.get(2)?,
                    started_at: row.get(3)?,
                    completed_at: row.get(4)?,
                    error: row.get(5)?,
                })
            })?;
            for row in rows {
                executions.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, workflow_id, status, started_at, completed_at, error
                 FROM workflow_executions
                 ORDER BY started_at DESC
                 LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(WorkflowExecution {
                    id: row.get(0)?,
                    workflow_id: row.get(1)?,
                    status: row.get(2)?,
                    started_at: row.get(3)?,
                    completed_at: row.get(4)?,
                    error: row.get(5)?,
                })
            })?;
            for row in rows {
                executions.push(row?);
            }
        }
        Ok(executions)
    }
}
