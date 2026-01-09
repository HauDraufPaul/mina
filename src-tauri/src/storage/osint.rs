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
    pub read: bool,
    pub favorite: bool,
    pub saved: bool,
    pub folder_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleFolder {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub id: i64,
    pub article_id: i64,
    pub entity_type: String,
    pub name: String,
    pub confidence: f64,
    pub context: Option<String>,
    pub extracted_at: i64,
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
    pub conn: Arc<Mutex<Connection>>,
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
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

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
                read INTEGER NOT NULL DEFAULT 0,
                favorite INTEGER NOT NULL DEFAULT 0,
                saved INTEGER NOT NULL DEFAULT 0,
                folder_id INTEGER,
                FOREIGN KEY (feed_id) REFERENCES rss_feeds(id),
                FOREIGN KEY (folder_id) REFERENCES article_folders(id)
            )",
            [],
        )?;

        // Migrate existing rss_items table
        let _ = conn.execute("ALTER TABLE rss_items ADD COLUMN read INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE rss_items ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE rss_items ADD COLUMN saved INTEGER NOT NULL DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE rss_items ADD COLUMN folder_id INTEGER", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS article_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS extracted_entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                article_id INTEGER NOT NULL,
                entity_type TEXT NOT NULL,
                name TEXT NOT NULL,
                confidence REAL NOT NULL,
                context TEXT,
                extracted_at INTEGER NOT NULL,
                FOREIGN KEY (article_id) REFERENCES rss_items(id) ON DELETE CASCADE,
                UNIQUE(article_id, entity_type, name)
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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rss_items_read ON rss_items(read)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rss_items_favorite ON rss_items(favorite)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rss_items_saved ON rss_items(saved)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_extracted_entities_article ON extracted_entities(article_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_extracted_entities_type ON extracted_entities(entity_type)",
            [],
        )?;

        // Don't initialize default feeds here - do it lazily on first access
        // This prevents hanging during app startup
        // Default feeds will be added when the first feed list is requested

        Ok(())
    }

    fn init_default_feeds(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
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
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
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
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
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

    pub fn update_feed_last_fetch(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute("UPDATE rss_feeds SET last_fetch = ?1 WHERE id = ?2", params![now, id])?;
        Ok(())
    }

    pub fn delete_feed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM rss_feeds WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<RSSFeed>> {
        // Initialize default feeds lazily on first access
        let _ = self.init_default_feeds();
        
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
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
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let fetched_at = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO rss_items (feed_id, title, content, url, published_at, fetched_at, read, favorite, saved)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 0)",
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
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        // Get items ordered by feed reliability and recency
        let mut stmt = conn.prepare(
            "SELECT i.id, i.feed_id, i.title, i.content, i.url, i.published_at, i.fetched_at, 
                    i.read, i.favorite, i.saved, i.folder_id
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
                read: row.get::<_, i64>(7)? == 1,
                favorite: row.get::<_, i64>(8)? == 1,
                saved: row.get::<_, i64>(9)? == 1,
                folder_id: row.get(10)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn get_item(&self, id: i64) -> Result<Option<RSSItem>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, content, url, published_at, fetched_at, read, favorite, saved, folder_id
             FROM rss_items WHERE id = ?1"
        )?;

        match stmt.query_row(params![id], |row| {
            Ok(RSSItem {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                url: row.get(4)?,
                published_at: row.get(5)?,
                fetched_at: row.get(6)?,
                read: row.get::<_, i64>(7)? == 1,
                favorite: row.get::<_, i64>(8)? == 1,
                saved: row.get::<_, i64>(9)? == 1,
                folder_id: row.get(10)?,
            })
        }) {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    pub fn mark_as_read(&self, id: i64, read: bool) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "UPDATE rss_items SET read = ?1 WHERE id = ?2",
            params![if read { 1i64 } else { 0i64 }, id],
        )?;
        Ok(())
    }

    pub fn toggle_favorite(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let current: i64 = conn.query_row(
            "SELECT favorite FROM rss_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let new_value = if current == 1 { 0 } else { 1 };
        conn.execute(
            "UPDATE rss_items SET favorite = ?1 WHERE id = ?2",
            params![new_value, id],
        )?;
        Ok(new_value == 1)
    }

    pub fn toggle_saved(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let current: i64 = conn.query_row(
            "SELECT saved FROM rss_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let new_value = if current == 1 { 0 } else { 1 };
        conn.execute(
            "UPDATE rss_items SET saved = ?1 WHERE id = ?2",
            params![new_value, id],
        )?;
        Ok(new_value == 1)
    }

    pub fn set_folder(&self, id: i64, folder_id: Option<i64>) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "UPDATE rss_items SET folder_id = ?1 WHERE id = ?2",
            params![folder_id, id],
        )?;
        Ok(())
    }

    pub fn create_folder(&self, name: &str, color: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO article_folders (name, color, created_at) VALUES (?1, ?2, ?3)",
            params![name, color, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_folders(&self) -> Result<Vec<ArticleFolder>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, color, created_at FROM article_folders ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ArticleFolder {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut folders = Vec::new();
        for row in rows {
            folders.push(row?);
        }
        Ok(folders)
    }

    pub fn delete_folder(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "UPDATE rss_items SET folder_id = NULL WHERE folder_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM article_folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_items_by_filter(
        &self,
        favorite: Option<bool>,
        saved: Option<bool>,
        read: Option<bool>,
        folder_id: Option<i64>,
        limit: i32,
    ) -> Result<Vec<RSSItem>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut query = "SELECT i.id, i.feed_id, i.title, i.content, i.url, i.published_at, i.fetched_at, 
                                i.read, i.favorite, i.saved, i.folder_id
                         FROM rss_items i
                         JOIN rss_feeds f ON i.feed_id = f.id
                         WHERE f.enabled = 1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(fav) = favorite {
            query.push_str(" AND i.favorite = ?");
            params_vec.push(Box::new(if fav { 1i64 } else { 0i64 }));
        }
        if let Some(sav) = saved {
            query.push_str(" AND i.saved = ?");
            params_vec.push(Box::new(if sav { 1i64 } else { 0i64 }));
        }
        if let Some(rd) = read {
            query.push_str(" AND i.read = ?");
            params_vec.push(Box::new(if rd { 1i64 } else { 0i64 }));
        }
        if let Some(fid) = folder_id {
            query.push_str(" AND i.folder_id = ?");
            params_vec.push(Box::new(fid));
        }

        query.push_str(" ORDER BY f.reliability DESC, i.published_at DESC LIMIT ?");
        params_vec.push(Box::new(limit));

        let mut params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(RSSItem {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                url: row.get(4)?,
                published_at: row.get(5)?,
                fetched_at: row.get(6)?,
                read: row.get::<_, i64>(7)? == 1,
                favorite: row.get::<_, i64>(8)? == 1,
                saved: row.get::<_, i64>(9)? == 1,
                folder_id: row.get(10)?,
            })
        })?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn save_extracted_entity(
        &self,
        article_id: i64,
        entity_type: &str,
        name: &str,
        confidence: f64,
        context: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR IGNORE INTO extracted_entities (article_id, entity_type, name, confidence, context, extracted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![article_id, entity_type, name, confidence, context, now],
        )?;

        let id: i64 = conn.query_row(
            "SELECT id FROM extracted_entities WHERE article_id = ?1 AND entity_type = ?2 AND name = ?3",
            params![article_id, entity_type, name],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn get_entities_for_article(&self, article_id: i64) -> Result<Vec<ExtractedEntity>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, article_id, entity_type, name, confidence, context, extracted_at
             FROM extracted_entities
             WHERE article_id = ?1
             ORDER BY confidence DESC, entity_type, name"
        )?;

        let rows = stmt.query_map(params![article_id], |row| {
            Ok(ExtractedEntity {
                id: row.get(0)?,
                article_id: row.get(1)?,
                entity_type: row.get(2)?,
                name: row.get(3)?,
                confidence: row.get(4)?,
                context: row.get(5)?,
                extracted_at: row.get(6)?,
            })
        })?;

        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
        }
        Ok(entities)
    }

    pub fn create_entity(&self, entity_type: &str, name: &str, metadata: &str) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO entities (entity_type, name, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![entity_type, name, metadata, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_entities(&self, entity_type: Option<&str>) -> Result<Vec<Entity>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
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
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO entity_relationships (source_id, target_id, relationship_type, strength, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![source_id, target_id, relationship_type, strength, now],
        )?;

        Ok(conn.last_insert_rowid())
    }
}
