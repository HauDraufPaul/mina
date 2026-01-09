use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingConversation {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub sender: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: i64,
    pub message_id: i64,
    pub attachment_type: String, // chart|event|ticker|link
    pub data_json: Value,
    pub created_at: i64,
}

pub struct MessagingStore {
    pub conn: Arc<Mutex<Connection>>,
}

impl MessagingStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = MessagingStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: MessagingStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                sender TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS message_attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL,
                attachment_type TEXT NOT NULL,
                data_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_attachments_message ON message_attachments(message_id)",
            [],
        )?;

        Ok(())
    }

    pub fn create_conversation(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO conversations (name, created_at, updated_at) VALUES (?1, ?2, ?3)",
            params![name, now, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_conversations(&self) -> Result<Vec<MessagingConversation>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(MessagingConversation {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;

        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(row?);
        }

        Ok(conversations)
    }

    pub fn send_message(
        &self,
        conversation_id: i64,
        sender: &str,
        content: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO messages (conversation_id, sender, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![conversation_id, sender, content, now],
        )?;

        // Update conversation updated_at
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, conversation_id],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_conversation_messages(&self, conversation_id: i64, limit: i64) -> Result<Vec<Message>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let limit = limit.max(1).min(1000);

        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, sender, content, created_at
             FROM messages
             WHERE conversation_id = ?1
             ORDER BY created_at ASC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![conversation_id, limit], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                sender: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        Ok(messages)
    }

    pub fn attach_to_message(
        &self,
        message_id: i64,
        attachment_type: &str,
        data_json: &Value,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO message_attachments (message_id, attachment_type, data_json, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![message_id, attachment_type, data_json.to_string(), now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_message_attachments(&self, message_id: i64) -> Result<Vec<MessageAttachment>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, message_id, attachment_type, data_json, created_at
             FROM message_attachments
             WHERE message_id = ?1
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![message_id], |row| {
            let data_json_str: String = row.get(3)?;
            let data_json: Value = serde_json::from_str(&data_json_str)
                .unwrap_or_else(|_| serde_json::json!({}));

            Ok(MessageAttachment {
                id: row.get(0)?,
                message_id: row.get(1)?,
                attachment_type: row.get(2)?,
                data_json,
                created_at: row.get(4)?,
            })
        })?;

        let mut attachments = Vec::new();
        for row in rows {
            attachments.push(row?);
        }

        Ok(attachments)
    }
}
