use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use std::sync::{Arc, Mutex};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

pub struct APIKeyStore {
    conn: Arc<Mutex<Connection>>,
    encryption_key: [u8; 32],
}

impl APIKeyStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self> {
        let store = APIKeyStore {
            conn,
            encryption_key: Self::derive_encryption_key()?,
        };
        store.init_schema()
            .context("Failed to initialize API key schema")?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_keys (
                provider TEXT PRIMARY KEY,
                encrypted_key TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_api_keys_provider ON api_keys(provider)",
            [],
        )?;

        Ok(())
    }

    /// Derive encryption key from system identifier
    /// In production, this should use a user-provided master password
    fn derive_encryption_key() -> Result<[u8; 32]> {
        // Use a system identifier (in production, use user PIN/password)
        let salt = b"mina_api_key_salt_v1"; // Fixed salt for now
        let mut key = [0u8; 32];
        
        // Derive key using PBKDF2
        // In production, use a user-provided password here
        let password = std::env::var("USER").unwrap_or_else(|_| "default".to_string());
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100000, &mut key);
        
        Ok(key)
    }

    /// Encrypt API key using AES-256-GCM
    fn encrypt_key(&self, key: &str) -> Result<String> {
        let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Combine nonce and ciphertext, then base64 encode
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);
        Ok(general_purpose::STANDARD.encode(&combined))
    }

    /// Decrypt API key
    fn decrypt_key(&self, encrypted: &str) -> Result<String> {
        let combined = general_purpose::STANDARD.decode(encrypted)
            .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))?;
        
        if combined.len() < 12 {
            anyhow::bail!("Invalid encrypted data");
        }
        
        let nonce = Nonce::from_slice(&combined[0..12]);
        let ciphertext = &combined[12..];
        
        let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&self.encryption_key));
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }

    /// Store an API key for a provider
    pub fn store_key(&self, provider: &str, key: &str) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        let encrypted = self.encrypt_key(key)?;
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT OR REPLACE INTO api_keys (provider, encrypted_key, created_at, updated_at)
             VALUES (?1, ?2, COALESCE((SELECT created_at FROM api_keys WHERE provider = ?1), ?3), ?3)",
            params![provider, encrypted, now, now],
        )?;
        
        Ok(())
    }

    /// Get API key for a provider
    pub fn get_key(&self, provider: &str) -> Result<Option<String>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        let encrypted: Option<String> = conn
            .query_row(
                "SELECT encrypted_key FROM api_keys WHERE provider = ?1",
                params![provider],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        
        match encrypted {
            Some(enc) => {
                let decrypted = self.decrypt_key(&enc)?;
                Ok(Some(decrypted))
            }
            None => Ok(None),
        }
    }

    /// Delete API key for a provider
    pub fn delete_key(&self, provider: &str) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        conn.execute(
            "DELETE FROM api_keys WHERE provider = ?1",
            params![provider],
        )?;
        
        Ok(())
    }

    /// List all providers with stored keys
    pub fn list_providers(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        let mut stmt = conn.prepare("SELECT provider FROM api_keys ORDER BY provider")?;
        let providers = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(providers)
    }

    /// Check if a provider has a stored key
    pub fn has_key(&self, provider: &str) -> Result<bool> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM api_keys WHERE provider = ?1",
            params![provider],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }
}

