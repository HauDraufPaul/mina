use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitBucket {
    pub name: String,
    pub capacity: i64,
    pub tokens: i64,
    pub refill_rate: i64,
    pub refill_interval: i64, // seconds
    pub last_refill: i64,
}

pub struct RateLimitStore {
    conn: Arc<Mutex<Connection>>,
}

impl RateLimitStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = RateLimitStore { conn };
        store.init_schema().unwrap();
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rate_limit_buckets (
                name TEXT PRIMARY KEY,
                capacity INTEGER NOT NULL,
                tokens INTEGER NOT NULL,
                refill_rate INTEGER NOT NULL,
                refill_interval INTEGER NOT NULL,
                last_refill INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_bucket(
        &self,
        name: &str,
        capacity: i64,
        refill_rate: i64,
        refill_interval: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO rate_limit_buckets 
             (name, capacity, tokens, refill_rate, refill_interval, last_refill)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, capacity, capacity, refill_rate, refill_interval, now],
        )?;

        Ok(())
    }

    pub fn get_bucket(&self, name: &str) -> Result<Option<RateLimitBucket>> {
        let conn = self.conn.lock().unwrap();
        
        let bucket: Option<RateLimitBucket> = conn
            .query_row(
                "SELECT name, capacity, tokens, refill_rate, refill_interval, last_refill
                 FROM rate_limit_buckets WHERE name = ?1",
                params![name],
                |row| {
                    Ok(RateLimitBucket {
                        name: row.get(0)?,
                        capacity: row.get(1)?,
                        tokens: row.get(2)?,
                        refill_rate: row.get(3)?,
                        refill_interval: row.get(4)?,
                        last_refill: row.get(5)?,
                    })
                },
            )
            .optional()?;

        Ok(bucket)
    }

    pub fn list_buckets(&self) -> Result<Vec<RateLimitBucket>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name, capacity, tokens, refill_rate, refill_interval, last_refill
             FROM rate_limit_buckets"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RateLimitBucket {
                name: row.get(0)?,
                capacity: row.get(1)?,
                tokens: row.get(2)?,
                refill_rate: row.get(3)?,
                refill_interval: row.get(4)?,
                last_refill: row.get(5)?,
            })
        })?;

        let mut buckets = Vec::new();
        for row in rows {
            buckets.push(row?);
        }
        Ok(buckets)
    }

    pub fn consume_token(&self, name: &str, amount: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Get bucket and refill if needed
        let bucket: Option<RateLimitBucket> = conn
            .query_row(
                "SELECT name, capacity, tokens, refill_rate, refill_interval, last_refill
                 FROM rate_limit_buckets WHERE name = ?1",
                params![name],
                |row| {
                    Ok(RateLimitBucket {
                        name: row.get(0)?,
                        capacity: row.get(1)?,
                        tokens: row.get(2)?,
                        refill_rate: row.get(3)?,
                        refill_interval: row.get(4)?,
                        last_refill: row.get(5)?,
                    })
                },
            )
            .optional()?;

        if let Some(mut bucket) = bucket {
            // Refill tokens if interval has passed
            let elapsed = now - bucket.last_refill;
            if elapsed >= bucket.refill_interval {
                let refills = elapsed / bucket.refill_interval;
                bucket.tokens = (bucket.tokens + bucket.refill_rate * refills).min(bucket.capacity);
                bucket.last_refill = now;
            }

            // Check if we have enough tokens
            if bucket.tokens >= amount {
                bucket.tokens -= amount;
                
                // Update bucket
                conn.execute(
                    "UPDATE rate_limit_buckets 
                     SET tokens = ?1, last_refill = ?2 
                     WHERE name = ?3",
                    params![bucket.tokens, bucket.last_refill, name],
                )?;
                
                Ok(true)
            } else {
                // Update last_refill even if we can't consume
                conn.execute(
                    "UPDATE rate_limit_buckets SET last_refill = ?1 WHERE name = ?2",
                    params![now, name],
                )?;
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn refill_bucket(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let bucket: Option<RateLimitBucket> = conn
            .query_row(
                "SELECT name, capacity, tokens, refill_rate, refill_interval, last_refill
                 FROM rate_limit_buckets WHERE name = ?1",
                params![name],
                |row| {
                    Ok(RateLimitBucket {
                        name: row.get(0)?,
                        capacity: row.get(1)?,
                        tokens: row.get(2)?,
                        refill_rate: row.get(3)?,
                        refill_interval: row.get(4)?,
                        last_refill: row.get(5)?,
                    })
                },
            )
            .optional()?;

        if let Some(mut bucket) = bucket {
            let elapsed = now - bucket.last_refill;
            if elapsed >= bucket.refill_interval {
                let refills = elapsed / bucket.refill_interval;
                bucket.tokens = (bucket.tokens + bucket.refill_rate * refills).min(bucket.capacity);
                bucket.last_refill = now;

                conn.execute(
                    "UPDATE rate_limit_buckets 
                     SET tokens = ?1, last_refill = ?2 
                     WHERE name = ?3",
                    params![bucket.tokens, bucket.last_refill, name],
                )?;
            }
        }

        Ok(())
    }
}

