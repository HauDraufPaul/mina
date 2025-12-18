use anyhow::Result;
use rusqlite::Connection;

pub struct MigrationManager {
    migrations: Vec<Migration>,
}

struct Migration {
    version: i32,
    name: String,
    up: &'static str,
    down: Option<&'static str>,
}

impl MigrationManager {
    pub fn new() -> Self {
        let mut migrations = Vec::new();

        // Migration 1: Initial schema
        migrations.push(Migration {
            version: 1,
            name: "initial_schema".to_string(),
            up: "CREATE TABLE IF NOT EXISTS migration_history (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )",
            down: None,
        });

        // Add more migrations here as needed
        // migrations.push(Migration { ... });

        MigrationManager { migrations }
    }

    pub fn migrate(&self, conn: &Connection) -> Result<()> {
        // Ensure migration_history table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS migration_history (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Get applied migrations
        let mut stmt = conn.prepare("SELECT version FROM migration_history ORDER BY version")?;
        let applied: Vec<i32> = stmt
            .query_map([], |row| Ok(row.get(0)?))?
            .collect::<Result<Vec<_>, _>>()?;

        let applied_set: std::collections::HashSet<i32> = applied.into_iter().collect();

        // Apply pending migrations
        for migration in &self.migrations {
            if !applied_set.contains(&migration.version) {
                conn.execute(migration.up, [])?;
                
                let timestamp = chrono::Utc::now().timestamp();
                conn.execute(
                    "INSERT INTO migration_history (version, name, applied_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![migration.version, migration.name, timestamp],
                )?;

                println!("Applied migration {}: {}", migration.version, migration.name);
            }
        }

        Ok(())
    }

    pub fn rollback(&self, _conn: &Connection, _version: i32) -> Result<()> {
        // Implement rollback logic if needed
        Ok(())
    }
}

