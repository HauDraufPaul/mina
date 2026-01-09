use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLayoutData {
    pub id: String,
    pub name: String,
    pub layout_json: String, // JSON string of the full layout
    pub columns: i32,
    pub rows: i32,
    pub is_template: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct GridLayoutStore {
    conn: Arc<Mutex<Connection>>,
}

impl GridLayoutStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = GridLayoutStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: GridLayoutStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS grid_layouts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                layout_json TEXT NOT NULL,
                columns INTEGER NOT NULL DEFAULT 2,
                rows INTEGER NOT NULL DEFAULT 2,
                is_template INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_grid_layouts_name ON grid_layouts(name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_grid_layouts_template ON grid_layouts(is_template)",
            [],
        )?;

        Ok(())
    }

    pub fn create_layout(
        &self,
        id: &str,
        name: &str,
        layout_json: &str,
        columns: i32,
        rows: i32,
        is_template: bool,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO grid_layouts (id, name, layout_json, columns, rows, is_template, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, layout_json, columns, rows, if is_template { 1 } else { 0 }, now, now],
        )?;

        Ok(())
    }

    pub fn update_layout(
        &self,
        id: &str,
        name: Option<&str>,
        layout_json: Option<&str>,
        columns: Option<i32>,
        rows: Option<i32>,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(n) = name {
            updates.push("name = ?");
            params_vec.push(Box::new(n));
        }
        if let Some(lj) = layout_json {
            updates.push("layout_json = ?");
            params_vec.push(Box::new(lj));
        }
        if let Some(c) = columns {
            updates.push("columns = ?");
            params_vec.push(Box::new(c));
        }
        if let Some(r) = rows {
            updates.push("rows = ?");
            params_vec.push(Box::new(r));
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        params_vec.push(Box::new(now));
        params_vec.push(Box::new(id));

        let sql = format!(
            "UPDATE grid_layouts SET {} WHERE id = ?",
            updates.join(", ")
        );

        let mut stmt = conn.prepare(&sql)?;
        stmt.execute(rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())))?;

        Ok(())
    }

    pub fn get_layout(&self, id: &str) -> Result<Option<GridLayoutData>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.query_row(
            "SELECT id, name, layout_json, columns, rows, is_template, created_at, updated_at
             FROM grid_layouts WHERE id = ?1",
            params![id],
            |row| {
                Ok(GridLayoutData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    layout_json: row.get(2)?,
                    columns: row.get(3)?,
                    rows: row.get(4)?,
                    is_template: row.get::<_, i32>(5)? == 1,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(anyhow::Error::from)
    }

    pub fn list_layouts(&self, include_templates: bool) -> Result<Vec<GridLayoutData>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        let sql = if include_templates {
            "SELECT id, name, layout_json, columns, rows, is_template, created_at, updated_at
             FROM grid_layouts ORDER BY updated_at DESC"
        } else {
            "SELECT id, name, layout_json, columns, rows, is_template, created_at, updated_at
             FROM grid_layouts WHERE is_template = 0 ORDER BY updated_at DESC"
        };

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(GridLayoutData {
                id: row.get(0)?,
                name: row.get(1)?,
                layout_json: row.get(2)?,
                columns: row.get(3)?,
                rows: row.get(4)?,
                is_template: row.get::<_, i32>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut layouts = Vec::new();
        for row in rows {
            layouts.push(row?);
        }

        Ok(layouts)
    }

    pub fn delete_layout(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute("DELETE FROM grid_layouts WHERE id = ?1", params![id])?;

        Ok(())
    }

    pub fn list_templates(&self) -> Result<Vec<GridLayoutData>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, layout_json, columns, rows, is_template, created_at, updated_at
             FROM grid_layouts WHERE is_template = 1 ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GridLayoutData {
                id: row.get(0)?,
                name: row.get(1)?,
                layout_json: row.get(2)?,
                columns: row.get(3)?,
                rows: row.get(4)?,
                is_template: row.get::<_, i32>(5)? == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut layouts = Vec::new();
        for row in rows {
            layouts.push(row?);
        }

        Ok(layouts)
    }
}

