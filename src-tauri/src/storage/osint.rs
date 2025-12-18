use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSSFeed {
    pub id: i64,
    pub url: String,
    pub name: String,
    pub enabled: bool,
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
        store.init_schema().unwrap();
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
                last_fetch INTEGER,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

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

        Ok(())
    }

    pub fn create_feed(&self, url: &str, name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO rss_feeds (url, name, enabled, created_at)
             VALUES (?1, ?2, 1, ?3)",
            params![url, name, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_feeds(&self) -> Result<Vec<RSSFeed>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, name, enabled, last_fetch, created_at FROM rss_feeds ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RSSFeed {
                id: row.get(0)?,
                url: row.get(1)?,
                name: row.get(2)?,
                enabled: row.get::<_, i64>(3)? == 1,
                last_fetch: row.get(4)?,
                created_at: row.get(5)?,
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

        Ok(conn.last_insert_rowid())
    }

    pub fn get_recent_items(&self, limit: i32) -> Result<Vec<RSSItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, content, url, published_at, fetched_at
             FROM rss_items
             ORDER BY published_at DESC
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
        let query = if entity_type.is_some() {
            "SELECT id, entity_type, name, metadata, created_at FROM entities WHERE entity_type = ?1 ORDER BY name"
        } else {
            "SELECT id, entity_type, name, metadata, created_at FROM entities ORDER BY name"
        };

        let mut stmt = if entity_type.is_some() {
            conn.prepare(query)?
        } else {
            conn.prepare(query)?
        };

        let rows = if let Some(et) = entity_type {
            stmt.query_map(params![et], |row| {
                Ok(Entity {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    metadata: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
        } else {
            stmt.query_map([], |row| {
                Ok(Entity {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    metadata: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
        };

        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
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

