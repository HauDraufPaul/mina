use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    pub id: i64,
    pub portfolio_id: i64,
    pub ticker: String,
    pub quantity: f64,
    pub purchase_price: f64,
    pub purchase_date: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub portfolio_id: i64,
    pub ticker: String,
    pub transaction_type: String, // buy|sell
    pub quantity: f64,
    pub price: f64,
    pub transaction_date: i64,
    pub fees: f64,
    pub notes: Option<String>,
}

pub struct PortfolioStore {
    pub conn: Arc<Mutex<Connection>>,
}

impl PortfolioStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = PortfolioStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: PortfolioStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS portfolios (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS holdings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                portfolio_id INTEGER NOT NULL,
                ticker TEXT NOT NULL,
                quantity REAL NOT NULL,
                purchase_price REAL NOT NULL,
                purchase_date INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE,
                UNIQUE(portfolio_id, ticker)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                portfolio_id INTEGER NOT NULL,
                ticker TEXT NOT NULL,
                transaction_type TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                transaction_date INTEGER NOT NULL,
                fees REAL NOT NULL DEFAULT 0.0,
                notes TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_holdings_portfolio ON holdings(portfolio_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_holdings_ticker ON holdings(ticker)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_portfolio ON transactions(portfolio_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transactions_ticker ON transactions(ticker)",
            [],
        )?;

        Ok(())
    }

    pub fn create_portfolio(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO portfolios (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_portfolios(&self) -> Result<Vec<Portfolio>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at FROM portfolios ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Portfolio {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;

        let mut portfolios = Vec::new();
        for row in rows {
            portfolios.push(row?);
        }

        Ok(portfolios)
    }

    pub fn get_portfolio(&self, id: i64) -> Result<Option<Portfolio>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        let portfolio = conn
            .query_row(
                "SELECT id, name, created_at FROM portfolios WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Portfolio {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        created_at: row.get(2)?,
                    })
                },
            )
            .optional()?;

        Ok(portfolio)
    }

    pub fn add_holding(
        &self,
        portfolio_id: i64,
        ticker: &str,
        quantity: f64,
        purchase_price: f64,
        purchase_date: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO holdings (portfolio_id, ticker, quantity, purchase_price, purchase_date, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![portfolio_id, ticker, quantity, purchase_price, purchase_date, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_holdings(&self, portfolio_id: i64) -> Result<Vec<Holding>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, portfolio_id, ticker, quantity, purchase_price, purchase_date
             FROM holdings
             WHERE portfolio_id = ?1
             ORDER BY ticker",
        )?;

        let rows = stmt.query_map(params![portfolio_id], |row| {
            Ok(Holding {
                id: row.get(0)?,
                portfolio_id: row.get(1)?,
                ticker: row.get(2)?,
                quantity: row.get(3)?,
                purchase_price: row.get(4)?,
                purchase_date: row.get(5)?,
            })
        })?;

        let mut holdings = Vec::new();
        for row in rows {
            holdings.push(row?);
        }

        Ok(holdings)
    }

    pub fn get_holdings_by_ticker(&self, ticker: &str) -> Result<Vec<Holding>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, portfolio_id, ticker, quantity, purchase_price, purchase_date
             FROM holdings
             WHERE ticker = ?1
             ORDER BY portfolio_id",
        )?;

        let rows = stmt.query_map(params![ticker], |row| {
            Ok(Holding {
                id: row.get(0)?,
                portfolio_id: row.get(1)?,
                ticker: row.get(2)?,
                quantity: row.get(3)?,
                purchase_price: row.get(4)?,
                purchase_date: row.get(5)?,
            })
        })?;

        let mut holdings = Vec::new();
        for row in rows {
            holdings.push(row?);
        }

        Ok(holdings)
    }

    pub fn update_holding(
        &self,
        holding_id: i64,
        quantity: Option<f64>,
        purchase_price: Option<f64>,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        if let Some(qty) = quantity {
            conn.execute(
                "UPDATE holdings SET quantity = ?1 WHERE id = ?2",
                params![qty, holding_id],
            )?;
        }

        if let Some(price) = purchase_price {
            conn.execute(
                "UPDATE holdings SET purchase_price = ?1 WHERE id = ?2",
                params![price, holding_id],
            )?;
        }

        Ok(())
    }

    pub fn delete_holding(&self, holding_id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM holdings WHERE id = ?1", params![holding_id])?;
        Ok(())
    }

    pub fn add_transaction(
        &self,
        portfolio_id: i64,
        ticker: &str,
        transaction_type: &str,
        quantity: f64,
        price: f64,
        transaction_date: i64,
        fees: f64,
        notes: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO transactions (portfolio_id, ticker, transaction_type, quantity, price, transaction_date, fees, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                portfolio_id,
                ticker,
                transaction_type,
                quantity,
                price,
                transaction_date,
                fees,
                notes,
                now
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_transactions(&self, portfolio_id: i64, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let limit = limit.unwrap_or(100).max(1).min(1000);

        let mut stmt = conn.prepare(
            "SELECT id, portfolio_id, ticker, transaction_type, quantity, price, transaction_date, fees, notes
             FROM transactions
             WHERE portfolio_id = ?1
             ORDER BY transaction_date DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![portfolio_id, limit], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                portfolio_id: row.get(1)?,
                ticker: row.get(2)?,
                transaction_type: row.get(3)?,
                quantity: row.get(4)?,
                price: row.get(5)?,
                transaction_date: row.get(6)?,
                fees: row.get(7)?,
                notes: row.get(8)?,
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    pub fn delete_portfolio(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM portfolios WHERE id = ?1", params![id])?;
        Ok(())
    }
}
