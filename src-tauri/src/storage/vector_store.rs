use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorDocument {
    pub id: String,
    pub collection: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

pub struct VectorStore {
    conn: Arc<Mutex<Connection>>,
}

impl VectorStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = VectorStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: VectorStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vector_collections (
                name TEXT PRIMARY KEY,
                dimension INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vector_documents (
                id TEXT PRIMARY KEY,
                collection TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL,
                metadata TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER,
                FOREIGN KEY (collection) REFERENCES vector_collections(name)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vector_documents_collection ON vector_documents(collection)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vector_documents_expires_at ON vector_documents(expires_at)",
            [],
        )?;

        Ok(())
    }

    pub fn create_collection(&self, name: &str, dimension: i32) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO vector_collections (name, dimension, created_at)
             VALUES (?1, ?2, ?3)",
            params![name, dimension, now],
        )?;

        Ok(())
    }

    pub fn list_collections(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT name FROM vector_collections")?;

        let names = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;
        let mut collections = Vec::new();
        for name in names {
            collections.push(name?);
        }

        Ok(collections)
    }

    pub fn insert_document(&self, doc: &VectorDocument) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        // Serialize embedding to JSON for storage
        let embedding_json = serde_json::to_string(&doc.embedding)?;
        let metadata_json = serde_json::to_string(&doc.metadata)?;

        conn.execute(
            "INSERT OR REPLACE INTO vector_documents 
             (id, collection, content, embedding, metadata, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                doc.id,
                doc.collection,
                doc.content,
                embedding_json,
                metadata_json,
                doc.created_at,
                doc.expires_at
            ],
        )?;

        Ok(())
    }

    pub fn search_similar(
        &self,
        collection: &str,
        query_embedding: &[f32],
        limit: i32,
        min_similarity: f32,
    ) -> Result<Vec<(VectorDocument, f32)>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        // Note: This is a simplified cosine similarity search
        // In production, you'd use a proper vector database like Qdrant
        let mut stmt = conn.prepare(
            "SELECT id, collection, content, embedding, metadata, created_at, expires_at
             FROM vector_documents
             WHERE collection = ?1 AND (expires_at IS NULL OR expires_at > ?2)
             LIMIT ?3"
        )?;

        let now = chrono::Utc::now().timestamp();
        let rows = stmt.query_map(params![collection, now, limit * 10], |row| {
            let embedding_json: String = row.get(3)?;
            let embedding: Vec<f32> = serde_json::from_str(&embedding_json)
                .map_err(|_| rusqlite::Error::InvalidColumnType(3, "TEXT".to_string(), rusqlite::types::Type::Text))?;

            Ok(VectorDocument {
                id: row.get(0)?,
                collection: row.get(1)?,
                content: row.get(2)?,
                embedding,
                metadata: serde_json::from_str(&row.get::<_, String>(4)?)
                    .map_err(|_| rusqlite::Error::InvalidColumnType(4, "TEXT".to_string(), rusqlite::types::Type::Text))?,
                created_at: row.get(5)?,
                expires_at: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            let doc = row?;
            let similarity = cosine_similarity(query_embedding, &doc.embedding);
            if similarity >= min_similarity {
                results.push((doc, similarity));
            }
        }

        // Sort by similarity descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit as usize);

        Ok(results)
    }

    pub fn delete_expired(&self) -> Result<usize> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        let count = conn.execute(
            "DELETE FROM vector_documents WHERE expires_at IS NOT NULL AND expires_at < ?1",
            params![now],
        )?;

        Ok(count)
    }

    pub fn get_collection_stats(&self, collection: &str) -> Result<CollectionStats> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vector_documents WHERE collection = ?1",
            params![collection],
            |row| row.get(0),
        )?;

        let expired: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vector_documents 
             WHERE collection = ?1 AND expires_at IS NOT NULL AND expires_at < ?2",
            params![collection, chrono::Utc::now().timestamp()],
            |row| row.get(0),
        )?;

        Ok(CollectionStats {
            total: count as usize,
            expired: expired as usize,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionStats {
    pub total: usize,
    pub expired: usize,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

