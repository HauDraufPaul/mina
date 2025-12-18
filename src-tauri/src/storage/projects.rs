use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub project_type: String, // "playground", "shader", "script", "game"
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct ProjectStore {
    conn: Arc<Mutex<Connection>>,
}

impl ProjectStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = ProjectStore { conn };
        store.init_schema().unwrap();
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                project_type TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_projects_type ON projects(project_type)",
            [],
        )?;

        Ok(())
    }

    pub fn create_project(&self, name: &str, project_type: &str, content: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO projects (name, project_type, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, project_type, content, now, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn update_project(&self, id: i64, name: Option<&str>, content: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        if name.is_some() && content.is_some() {
            conn.execute(
                "UPDATE projects SET name = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                params![name.unwrap(), content.unwrap(), now, id],
            )?;
        } else if name.is_some() {
            conn.execute(
                "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name.unwrap(), now, id],
            )?;
        } else if content.is_some() {
            conn.execute(
                "UPDATE projects SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![content.unwrap(), now, id],
            )?;
        }

        Ok(())
    }

    pub fn list_projects(&self, project_type: Option<&str>) -> Result<Vec<Project>> {
        let conn = self.conn.lock().unwrap();
        let mut projects = Vec::new();
        
        if let Some(pt) = project_type {
            let mut stmt = conn.prepare(
                "SELECT id, name, project_type, content, created_at, updated_at FROM projects WHERE project_type = ?1 ORDER BY updated_at DESC"
            )?;
            let rows = stmt.query_map(params![pt], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    project_type: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;
            for row in rows {
                projects.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, name, project_type, content, created_at, updated_at FROM projects ORDER BY updated_at DESC"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    project_type: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;
            for row in rows {
                projects.push(row?);
            }
        }
        Ok(projects)
    }

    pub fn get_project(&self, id: i64) -> Result<Option<Project>> {
        let conn = self.conn.lock().unwrap();
        
        let project: Option<Project> = conn
            .query_row(
                "SELECT id, name, project_type, content, created_at, updated_at FROM projects WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        project_type: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()?;

        Ok(project)
    }

    pub fn delete_project(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }
}

