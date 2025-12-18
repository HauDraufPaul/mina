use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSSFeed {
    pub id: i64,
    pub url: String,
    pub name: String,
    pub enabled: bool,
    pub reliability: f64,
    pub last_fetch: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSSItem {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub content: String,
    pub url: String,
    pub published_at: i64,
    pub fetched_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: i64,
    pub entity_type: String, // "person", "organization", "location", "event"
    pub name: String,
    pub metadata: String, // JSON
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub relationship_type: String,
    pub strength: f64,
    pub created_at: i64,
}

pub struct OSINTStore {
    conn: Arc<Mutex<Connection>>,
}

impl OSINTStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = OSINTStore { conn };
        // Don't panic on schema initialization errors - log and continue
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: OSINTStore schema initialization failed: {}", e);
            eprintln!("WARNING: Continuing anyway - store may not function correctly");
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rss_feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                reliability REAL NOT NULL DEFAULT 0.5,
                last_fetch INTEGER,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Migrate existing tables to add reliability column if it doesn't exist
        // Ignore errors - column might already exist or table might be new
        // This is safe because CREATE TABLE already includes the column
        drop(conn.execute(
            "ALTER TABLE rss_feeds ADD COLUMN reliability REAL NOT NULL DEFAULT 0.5",
            [],
        ));

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rss_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                published_at INTEGER NOT NULL,
                fetched_at INTEGER NOT NULL,
                FOREIGN KEY (feed_id) REFERENCES rss_feeds(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                name TEXT NOT NULL,
                metadata TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS entity_relationships (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                target_id INTEGER NOT NULL,
                relationship_type TEXT NOT NULL,
                strength REAL NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (source_id) REFERENCES entities(id),
                FOREIGN KEY (target_id) REFERENCES entities(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rss_items_feed ON rss_items(feed_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_relationships_source ON entity_relationships(source_id)",
            [],
        )?;

        // Don't initialize default feeds here - do it lazily on first access
        // This prevents hanging during app startup
        // Default feeds will be added when the first feed list is requested

        Ok(())
    }

    fn init_default_feeds(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Check if any feeds exist (ignore errors if table doesn't exist yet)
        let count: i64 = match conn.query_row(
            "SELECT COUNT(*) FROM rss_feeds",
            [],
            |row| row.get(0),
        ) {
            Ok(c) => c,
            Err(_) => return Ok(()), // Table doesn't exist yet, skip initialization
        };

        // Only add defaults if no feeds exist
        if count == 0 {
            eprintln!("MINA: Adding default RSS feeds...");
            let default_feeds = vec![
                ("Hacker News", "https://hnrss.org/frontpage", 0.9),
                ("The Hacker News", "https://feeds.feedburner.com/TheHackersNews", 0.85),
                ("Ars Technica", "https://feeds.arstechnica.com/arstechnica/index", 0.95),
                ("TechCrunch", "https://techcrunch.com/feed/", 0.9),
                ("Wired", "https://www.wired.com/feed/rss", 0.85),
            ];

            let now = chrono::Utc::now().timestamp();
            for (name, url, reliability) in default_feeds {
                if let Err(e) = conn.execute(
                    "INSERT OR IGNORE INTO rss_feeds (url, name, enabled, reliability, created_at)
                     VALUES (?1, ?2, 1, ?3, ?4)",
                    params![url, name, reliability, now],
                ) {
                    eprintln!("Warning: Failed to insert default feed {}: {}", name, e);
                } else {
                    eprintln!("MINA: Added default feed: {}", name);
                }
            }
            eprintln!("MINA: Default feeds initialization complete");
        }

        Ok(())
    }

    pub fn create_feed(&self, url: &str, name: &str, reliability: Option<f64>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let rel = reliability.unwrap_or(0.5);

        conn.execute(
            "INSERT OR IGNORE INTO rss_feeds (url, name, enabled, reliability, created_at)
             VALUES (?1, ?2, 1, ?3, ?4)",
            params![url, name, rel, now],
        )?;

        // Get the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM rss_feeds WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn update_feed(&self, id: i64, name: Option<&str>, url: Option<&str>, reliability: Option<f64>, enabled: Option<bool>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        if let Some(n) = name {
            conn.execute("UPDATE rss_feeds SET name = ?1 WHERE id = ?2", params![n, id])?;
        }
        if let Some(u) = url {
            conn.execute("UPDATE rss_feeds SET url = ?1 WHERE id = ?2", params![u, id])?;
        }
        if let Some(r) = reliability {
            conn.execute("UPDATE rss_feeds SET reliability = ?1 WHERE id = ?2", params![r, id])?;
        }
        if let Some(e) = enabled {
            conn.execute("UPDATE rss_feeds SET enabled = ?1 WHERE id = ?2", params![if e { 1i64 } else { 0i64 }, id])?;
        }

        Ok(())
    }

    pub fn delete_feed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM rss_feeds WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<RSSFeed>> {
        // Initialize default feeds lazily on first access
        let _ = self.init_default_feeds();
        
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, name, enabled, reliability, last_fetch, created_at FROM rss_feeds ORDER BY reliability DESC, name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RSSFeed {
                id: row.get(0)?,
                url: row.get(1)?,
                name: row.get(2)?,
                enabled: row.get::<_, i64>(3)? == 1,
                reliability: row.get(4)?,
                last_fetch: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;

        let mut feeds = Vec::new();
        for row in rows {
            feeds.push(row?);
        }
        Ok(feeds)
    }

    pub fn save_rss_item(
        &self,
        feed_id: i64,
        title: &str,
        content: &str,
        url: &str,
        published_at: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let fetched_at = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO rss_items (feed_id, title, content, url, published_at, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![feed_id, title, content, url, published_at, fetched_at],
        )?;

        // Get the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM rss_items WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn get_recent_items(&self, limit: i32) -> Result<Vec<RSSItem>> {
        let conn = self.conn.lock().unwrap();
        // Get items ordered by feed reliability and recency
        let mut stmt = conn.prepare(
            "SELECT i.id, i.feed_id, i.title, i.content, i.url, i.published_at, i.fetched_at
             FROM rss_items i
             JOIN rss_feeds f ON i.feed_id = f.id
             WHERE f.enabled = 1
             ORDER BY f.reliability DESC, i.published_at DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(RSSItem {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                url: row.get(4)?,
                published_at: row.get(5)?,
                fetched_at: row.get(6)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn create_entity(&self, entity_type: &str, name: &str, metadata: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO entities (entity_type, name, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![entity_type, name, metadata, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_entities(&self, entity_type: Option<&str>) -> Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut entities = Vec::new();
        
        if let Some(et) = entity_type {
            let mut stmt = conn.prepare(
                "SELECT id, entity_type, name, metadata, created_at FROM entities WHERE entity_type = ?1 ORDER BY name"
            )?;
            let rows = stmt.query_map(params![et], |row| {
                Ok(Entity {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    metadata: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            for row in rows {
                entities.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, entity_type, name, metadata, created_at FROM entities ORDER BY name"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(Entity {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    metadata: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            for row in rows {
                entities.push(row?);
            }
        }
        Ok(entities)
    }

    pub fn create_relationship(
        &self,
        source_id: i64,
        target_id: i64,
        relationship_type: &str,
        strength: f64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO entity_relationships (source_id, target_id, relationship_type, strength, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![source_id, target_id, relationship_type, strength, now],
        )?;

        Ok(conn.last_insert_rowid())
    }
}
