use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i64,
    pub conversation_id: String,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: i64,
    pub model: Option<String>,
    pub tokens: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: i64,
    pub name: String,
    pub template: String,
    pub description: Option<String>,
    pub created_at: i64,
}

pub struct AIStore {
    conn: Arc<Mutex<Connection>>,
}

impl AIStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = AIStore { conn };
        store.init_schema().unwrap();
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                model TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                model TEXT,
                tokens INTEGER,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS prompt_templates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                template TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_conversation ON chat_messages(conversation_id)",
            [],
        )?;

        Ok(())
    }

    pub fn create_conversation(&self, id: &str, title: &str, model: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO conversations (id, title, created_at, updated_at, model)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, title, now, now, model],
        )?;

        Ok(())
    }

    pub fn list_conversations(&self) -> Result<Vec<Conversation>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at, model FROM conversations ORDER BY updated_at DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                model: row.get(4)?,
            })
        })?;

        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(row?);
        }
        Ok(conversations)
    }

    pub fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        model: Option<&str>,
        tokens: Option<i32>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO chat_messages (conversation_id, role, content, timestamp, model, tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![conversation_id, role, content, timestamp, model, tokens],
        )?;

        // Update conversation updated_at
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![timestamp, conversation_id],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, timestamp, model, tokens
             FROM chat_messages
             WHERE conversation_id = ?1
             ORDER BY timestamp ASC"
        )?;

        let rows = stmt.query_map(params![conversation_id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
                model: row.get(5)?,
                tokens: row.get(6)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn create_template(&self, name: &str, template: &str, description: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO prompt_templates (name, template, description, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, template, description, timestamp],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_templates(&self) -> Result<Vec<PromptTemplate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, template, description, created_at FROM prompt_templates ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                template: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(row?);
        }
        Ok(templates)
    }

    pub fn get_template(&self, name: &str) -> Result<Option<PromptTemplate>> {
        let conn = self.conn.lock().unwrap();
        
        let template: Option<PromptTemplate> = conn
            .query_row(
                "SELECT id, name, template, description, created_at FROM prompt_templates WHERE name = ?1",
                params![name],
                |row| {
                    Ok(PromptTemplate {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        template: row.get(2)?,
                        description: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .optional()?;

        Ok(template)
    }
}

