use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockTicker {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub index_name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockNewsItem {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub url: String,
    pub source: String,
    pub source_id: Option<String>,
    pub published_at: i64,
    pub fetched_at: i64,
    pub sentiment: Option<f64>,
    pub relevance_score: f64,
    pub tickers: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockNewsTicker {
    pub news_id: i64,
    pub ticker: String,
    pub confidence: f64,
}

pub struct StockNewsStore {
    pub conn: Arc<Mutex<Connection>>,
}

impl StockNewsStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        StockNewsStore { conn }
    }

    pub fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Stock tickers table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stock_tickers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                exchange TEXT NOT NULL,
                index_name TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Stock news table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stock_news (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                url TEXT UNIQUE NOT NULL,
                source TEXT NOT NULL,
                source_id TEXT,
                published_at INTEGER NOT NULL,
                fetched_at INTEGER NOT NULL,
                sentiment REAL,
                relevance_score REAL DEFAULT 1.0,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Stock news tickers association table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stock_news_tickers (
                news_id INTEGER NOT NULL,
                ticker TEXT NOT NULL,
                confidence REAL DEFAULT 1.0,
                FOREIGN KEY (news_id) REFERENCES stock_news(id),
                PRIMARY KEY (news_id, ticker)
            )",
            [],
        )?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stock_news_published ON stock_news(published_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stock_news_tickers_ticker ON stock_news_tickers(ticker)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stock_news_source ON stock_news(source)",
            [],
        )?;

        Ok(())
    }

    // Stock Ticker CRUD operations
    pub fn create_ticker(&self, symbol: &str, name: &str, exchange: &str, index_name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO stock_tickers (symbol, name, exchange, index_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![symbol, name, exchange, index_name, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_ticker(&self, symbol: &str) -> Result<Option<StockTicker>> {
        let conn = self.conn.lock().unwrap();
        
        conn.query_row(
            "SELECT id, symbol, name, exchange, index_name, created_at FROM stock_tickers WHERE symbol = ?1",
            params![symbol],
            |row| {
                Ok(StockTicker {
                    id: row.get(0)?,
                    symbol: row.get(1)?,
                    name: row.get(2)?,
                    exchange: row.get(3)?,
                    index_name: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_tickers(&self, index_name: Option<&str>) -> Result<Vec<StockTicker>> {
        let conn = self.conn.lock().unwrap();

        let query = if let Some(index) = index_name {
            format!(
                "SELECT id, symbol, name, exchange, index_name, created_at FROM stock_tickers WHERE index_name = '{}' ORDER BY symbol",
                index
            )
        } else {
            "SELECT id, symbol, name, exchange, index_name, created_at FROM stock_tickers ORDER BY symbol".to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            Ok(StockTicker {
                id: row.get(0)?,
                symbol: row.get(1)?,
                name: row.get(2)?,
                exchange: row.get(3)?,
                index_name: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        let mut tickers = Vec::new();
        for row in rows {
            tickers.push(row?);
        }
        Ok(tickers)
    }

    pub fn delete_ticker(&self, symbol: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM stock_tickers WHERE symbol = ?1", params![symbol])?;
        Ok(())
    }

    // Stock News CRUD operations
    pub fn create_news_item(
        &self,
        title: &str,
        content: &str,
        url: &str,
        source: &str,
        source_id: Option<&str>,
        published_at: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO stock_news 
             (title, content, url, source, source_id, published_at, fetched_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![title, content, url, source, source_id, published_at, now, now],
        )?;

        // Get the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM stock_news WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn create_news_item_with_sentiment(
        &self,
        title: &str,
        content: &str,
        url: &str,
        source: &str,
        source_id: Option<&str>,
        published_at: i64,
        sentiment: f64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO stock_news 
             (title, content, url, source, source_id, published_at, fetched_at, sentiment, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![title, content, url, source, source_id, published_at, now, sentiment, now],
        )?;

        // Get the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM stock_news WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn get_news_item(&self, id: i64) -> Result<Option<StockNewsItem>> {
        let conn = self.conn.lock().unwrap();

        let news = conn
            .query_row(
                "SELECT id, title, content, url, source, source_id, published_at, fetched_at, 
                        sentiment, relevance_score, created_at
                 FROM stock_news WHERE id = ?1",
                params![id],
                |row| {
                    Ok(StockNewsItem {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        content: row.get(2)?,
                        url: row.get(3)?,
                        source: row.get(4)?,
                        source_id: row.get(5)?,
                        published_at: row.get(6)?,
                        fetched_at: row.get(7)?,
                        sentiment: row.get(8)?,
                        relevance_score: row.get(9)?,
                        created_at: row.get(10)?,
                        tickers: Vec::new(), // Will be populated separately
                    })
                },
            )
            .optional()?;

        if let Some(mut news) = news {
            // Get associated tickers
            news.tickers = self.get_news_tickers(news.id)?;
            Ok(Some(news))
        } else {
            Ok(None)
        }
    }

    pub fn get_news(&self, tickers: Option<Vec<String>>, limit: i32, since: Option<i64>) -> Result<Vec<StockNewsItem>> {
        let conn = self.conn.lock().unwrap();

        let query = if let Some(ref ticker_list) = tickers {
            let ticker_placeholders = ticker_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            if let Some(_since_ts) = since {
                format!(
                    "SELECT DISTINCT n.id, n.title, n.content, n.url, n.source, n.source_id, 
                            n.published_at, n.fetched_at, n.sentiment, n.relevance_score, n.created_at
                     FROM stock_news n
                     JOIN stock_news_tickers nt ON n.id = nt.news_id
                     WHERE nt.ticker IN ({}) AND n.published_at >= ?
                     ORDER BY n.published_at DESC
                     LIMIT ?",
                    ticker_placeholders
                )
            } else {
                format!(
                    "SELECT DISTINCT n.id, n.title, n.content, n.url, n.source, n.source_id, 
                            n.published_at, n.fetched_at, n.sentiment, n.relevance_score, n.created_at
                     FROM stock_news n
                     JOIN stock_news_tickers nt ON n.id = nt.news_id
                     WHERE nt.ticker IN ({})
                     ORDER BY n.published_at DESC
                     LIMIT ?",
                    ticker_placeholders
                )
            }
        } else if let Some(_since_ts) = since {
            "SELECT id, title, content, url, source, source_id, published_at, fetched_at, 
                    sentiment, relevance_score, created_at
             FROM stock_news
             WHERE published_at >= ?
             ORDER BY published_at DESC
             LIMIT ?".to_string()
        } else {
            "SELECT id, title, content, url, source, source_id, published_at, fetched_at, 
                    sentiment, relevance_score, created_at
             FROM stock_news
             ORDER BY published_at DESC
             LIMIT ?".to_string()
        };

        let mut stmt = conn.prepare(&query)?;

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(ref ticker_list) = tickers {
            for ticker in ticker_list {
                params_vec.push(Box::new(ticker.clone()));
            }
        }
        if let Some(since_ts) = since {
            params_vec.push(Box::new(since_ts));
        }
        params_vec.push(Box::new(limit));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(StockNewsItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                published_at: row.get(6)?,
                fetched_at: row.get(7)?,
                sentiment: row.get(8)?,
                relevance_score: row.get(9)?,
                created_at: row.get(10)?,
                tickers: Vec::new(),
            })
        })?;

        // Collect items first
        let mut news_items = Vec::new();
        for row in rows {
            news_items.push(row?);
        }

        // Now get tickers for each item (can't call self.get_news_tickers while holding the lock)
        // So we need to get them directly here
        for item in &mut news_items {
            let mut ticker_stmt = conn.prepare(
                "SELECT ticker FROM stock_news_tickers WHERE news_id = ?1 ORDER BY confidence DESC",
            )?;
            let ticker_rows = ticker_stmt.query_map(params![item.id], |row| row.get(0))?;
            let mut item_tickers = Vec::new();
            for ticker_row in ticker_rows {
                item_tickers.push(ticker_row?);
            }
            item.tickers = item_tickers;
        }

        Ok(news_items)
    }

    pub fn search_news(&self, query: &str, tickers: Option<Vec<String>>, limit: i32) -> Result<Vec<StockNewsItem>> {
        let conn = self.conn.lock().unwrap();
        let search_term = format!("%{}%", query);

        let sql = if let Some(ref ticker_list) = tickers {
            let ticker_placeholders = ticker_list.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            format!(
                "SELECT DISTINCT n.id, n.title, n.content, n.url, n.source, n.source_id, 
                        n.published_at, n.fetched_at, n.sentiment, n.relevance_score, n.created_at
                 FROM stock_news n
                 JOIN stock_news_tickers nt ON n.id = nt.news_id
                 WHERE (n.title LIKE ? OR n.content LIKE ?) AND nt.ticker IN ({})
                 ORDER BY n.published_at DESC
                 LIMIT ?",
                ticker_placeholders
            )
        } else {
            "SELECT id, title, content, url, source, source_id, published_at, fetched_at, 
                    sentiment, relevance_score, created_at
             FROM stock_news
             WHERE title LIKE ? OR content LIKE ?
             ORDER BY published_at DESC
             LIMIT ?".to_string()
        };

        let mut stmt = conn.prepare(&sql)?;

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params_vec.push(Box::new(search_term.clone()));
        params_vec.push(Box::new(search_term));
        if let Some(ref ticker_list) = tickers {
            for ticker in ticker_list {
                params_vec.push(Box::new(ticker.clone()));
            }
        }
        params_vec.push(Box::new(limit));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(StockNewsItem {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                url: row.get(3)?,
                source: row.get(4)?,
                source_id: row.get(5)?,
                published_at: row.get(6)?,
                fetched_at: row.get(7)?,
                sentiment: row.get(8)?,
                relevance_score: row.get(9)?,
                created_at: row.get(10)?,
                tickers: Vec::new(),
            })
        })?;

        // Collect items first
        let mut news_items = Vec::new();
        for row in rows {
            news_items.push(row?);
        }

        // Now get tickers for each item
        for item in &mut news_items {
            let mut ticker_stmt = conn.prepare(
                "SELECT ticker FROM stock_news_tickers WHERE news_id = ?1 ORDER BY confidence DESC",
            )?;
            let ticker_rows = ticker_stmt.query_map(params![item.id], |row| row.get(0))?;
            let mut item_tickers = Vec::new();
            for ticker_row in ticker_rows {
                item_tickers.push(ticker_row?);
            }
            item.tickers = item_tickers;
        }

        Ok(news_items)
    }

    pub fn associate_ticker(&self, news_id: i64, ticker: &str, confidence: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO stock_news_tickers (news_id, ticker, confidence)
             VALUES (?1, ?2, ?3)",
            params![news_id, ticker, confidence],
        )?;
        Ok(())
    }

    pub fn get_news_tickers(&self, news_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ticker FROM stock_news_tickers WHERE news_id = ?1 ORDER BY confidence DESC",
        )?;

        let rows = stmt.query_map(params![news_id], |row| row.get(0))?;

        let mut tickers = Vec::new();
        for row in rows {
            tickers.push(row?);
        }
        Ok(tickers)
    }

    pub fn cleanup_old_news(&self, days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let cutoff = chrono::Utc::now().timestamp() - (days * 24 * 3600);

        let deleted = conn.execute(
            "DELETE FROM stock_news WHERE published_at < ?1",
            params![cutoff],
        )?;

        Ok(deleted)
    }

    // Initialize S&P 500 and DAX tickers
    pub fn init_default_tickers(&self) -> Result<()> {
        let sp500_tickers = get_sp500_tickers();
        let dax_tickers = get_dax_tickers();

        for (symbol, name, exchange) in sp500_tickers {
            let _ = self.create_ticker(&symbol, &name, &exchange, "SP500");
        }

        for (symbol, name, exchange) in dax_tickers {
            let _ = self.create_ticker(&symbol, &name, &exchange, "DAX");
        }

        Ok(())
    }
}

// S&P 500 tickers (top 50 for initial implementation, can be expanded)
fn get_sp500_tickers() -> Vec<(String, String, String)> {
    vec![
        ("AAPL".to_string(), "Apple Inc.".to_string(), "NASDAQ".to_string()),
        ("MSFT".to_string(), "Microsoft Corporation".to_string(), "NASDAQ".to_string()),
        ("GOOGL".to_string(), "Alphabet Inc. Class A".to_string(), "NASDAQ".to_string()),
        ("AMZN".to_string(), "Amazon.com Inc.".to_string(), "NASDAQ".to_string()),
        ("NVDA".to_string(), "NVIDIA Corporation".to_string(), "NASDAQ".to_string()),
        ("META".to_string(), "Meta Platforms Inc.".to_string(), "NASDAQ".to_string()),
        ("TSLA".to_string(), "Tesla Inc.".to_string(), "NASDAQ".to_string()),
        ("BRK.B".to_string(), "Berkshire Hathaway Inc. Class B".to_string(), "NYSE".to_string()),
        ("UNH".to_string(), "UnitedHealth Group Inc.".to_string(), "NYSE".to_string()),
        ("JNJ".to_string(), "Johnson & Johnson".to_string(), "NYSE".to_string()),
        ("XOM".to_string(), "Exxon Mobil Corporation".to_string(), "NYSE".to_string()),
        ("V".to_string(), "Visa Inc.".to_string(), "NYSE".to_string()),
        ("PG".to_string(), "Procter & Gamble Co.".to_string(), "NYSE".to_string()),
        ("JPM".to_string(), "JPMorgan Chase & Co.".to_string(), "NYSE".to_string()),
        ("MA".to_string(), "Mastercard Inc.".to_string(), "NYSE".to_string()),
        ("HD".to_string(), "The Home Depot Inc.".to_string(), "NYSE".to_string()),
        ("CVX".to_string(), "Chevron Corporation".to_string(), "NYSE".to_string()),
        ("MRK".to_string(), "Merck & Co. Inc.".to_string(), "NYSE".to_string()),
        ("ABBV".to_string(), "AbbVie Inc.".to_string(), "NYSE".to_string()),
        ("PEP".to_string(), "PepsiCo Inc.".to_string(), "NASDAQ".to_string()),
        ("KO".to_string(), "The Coca-Cola Company".to_string(), "NYSE".to_string()),
        ("COST".to_string(), "Costco Wholesale Corporation".to_string(), "NASDAQ".to_string()),
        ("AVGO".to_string(), "Broadcom Inc.".to_string(), "NASDAQ".to_string()),
        ("TMO".to_string(), "Thermo Fisher Scientific Inc.".to_string(), "NYSE".to_string()),
        ("WMT".to_string(), "Walmart Inc.".to_string(), "NYSE".to_string()),
        ("MCD".to_string(), "McDonald's Corporation".to_string(), "NYSE".to_string()),
        ("DIS".to_string(), "The Walt Disney Company".to_string(), "NYSE".to_string()),
        ("ABT".to_string(), "Abbott Laboratories".to_string(), "NYSE".to_string()),
        ("CSCO".to_string(), "Cisco Systems Inc.".to_string(), "NASDAQ".to_string()),
        ("CRM".to_string(), "Salesforce Inc.".to_string(), "NYSE".to_string()),
        ("ACN".to_string(), "Accenture plc".to_string(), "NYSE".to_string()),
        ("ADBE".to_string(), "Adobe Inc.".to_string(), "NASDAQ".to_string()),
        ("NFLX".to_string(), "Netflix Inc.".to_string(), "NASDAQ".to_string()),
        ("NKE".to_string(), "NIKE Inc.".to_string(), "NYSE".to_string()),
        ("CMCSA".to_string(), "Comcast Corporation".to_string(), "NASDAQ".to_string()),
        ("VZ".to_string(), "Verizon Communications Inc.".to_string(), "NYSE".to_string()),
        ("INTC".to_string(), "Intel Corporation".to_string(), "NASDAQ".to_string()),
        ("T".to_string(), "AT&T Inc.".to_string(), "NYSE".to_string()),
        ("PFE".to_string(), "Pfizer Inc.".to_string(), "NYSE".to_string()),
        ("WFC".to_string(), "Wells Fargo & Company".to_string(), "NYSE".to_string()),
        ("BAC".to_string(), "Bank of America Corporation".to_string(), "NYSE".to_string()),
        ("COP".to_string(), "ConocoPhillips".to_string(), "NYSE".to_string()),
        ("LLY".to_string(), "Eli Lilly and Company".to_string(), "NYSE".to_string()),
        ("ORCL".to_string(), "Oracle Corporation".to_string(), "NYSE".to_string()),
        ("AMD".to_string(), "Advanced Micro Devices Inc.".to_string(), "NASDAQ".to_string()),
        ("QCOM".to_string(), "QUALCOMM Inc.".to_string(), "NASDAQ".to_string()),
        ("TXN".to_string(), "Texas Instruments Inc.".to_string(), "NASDAQ".to_string()),
        ("RTX".to_string(), "RTX Corporation".to_string(), "NYSE".to_string()),
        ("IBM".to_string(), "International Business Machines Corporation".to_string(), "NYSE".to_string()),
        ("GE".to_string(), "General Electric Company".to_string(), "NYSE".to_string()),
    ]
}

// DAX tickers (40 stocks)
fn get_dax_tickers() -> Vec<(String, String, String)> {
    vec![
        ("SAP".to_string(), "SAP SE".to_string(), "XETR".to_string()),
        ("SIE".to_string(), "Siemens AG".to_string(), "XETR".to_string()),
        ("AIR".to_string(), "Airbus SE".to_string(), "XETR".to_string()),
        ("DTE".to_string(), "Deutsche Telekom AG".to_string(), "XETR".to_string()),
        ("ALV".to_string(), "Allianz SE".to_string(), "XETR".to_string()),
        ("MBG".to_string(), "Mercedes-Benz Group AG".to_string(), "XETR".to_string()),
        ("BMW".to_string(), "Bayerische Motoren Werke AG".to_string(), "XETR".to_string()),
        ("VOW3".to_string(), "Volkswagen AG".to_string(), "XETR".to_string()),
        ("BAS".to_string(), "BASF SE".to_string(), "XETR".to_string()),
        ("DB1".to_string(), "Deutsche Börse AG".to_string(), "XETR".to_string()),
        ("MUV2".to_string(), "Munich Re".to_string(), "XETR".to_string()),
        ("ADS".to_string(), "Adidas AG".to_string(), "XETR".to_string()),
        ("DBK".to_string(), "Deutsche Bank AG".to_string(), "XETR".to_string()),
        ("LIN".to_string(), "Linde plc".to_string(), "XETR".to_string()),
        ("BNR".to_string(), "Brenntag SE".to_string(), "XETR".to_string()),
        ("HEN3".to_string(), "Henkel AG & Co. KGaA".to_string(), "XETR".to_string()),
        ("IFX".to_string(), "Infineon Technologies AG".to_string(), "XETR".to_string()),
        ("RWE".to_string(), "RWE AG".to_string(), "XETR".to_string()),
        ("VON".to_string(), "Vonovia SE".to_string(), "XETR".to_string()),
        ("1COV".to_string(), "Covestro AG".to_string(), "XETR".to_string()),
        ("ZAL".to_string(), "Zalando SE".to_string(), "XETR".to_string()),
        ("HEI".to_string(), "HeidelbergCement AG".to_string(), "XETR".to_string()),
        ("FRE".to_string(), "Fresenius SE & Co. KGaA".to_string(), "XETR".to_string()),
        ("FME".to_string(), "Fresenius Medical Care AG & Co. KGaA".to_string(), "XETR".to_string()),
        ("CON".to_string(), "Continental AG".to_string(), "XETR".to_string()),
        ("BEI".to_string(), "Beiersdorf AG".to_string(), "XETR".to_string()),
        ("EOAN".to_string(), "E.ON SE".to_string(), "XETR".to_string()),
        ("DHL".to_string(), "Deutsche Post AG".to_string(), "XETR".to_string()),
        ("HNR1".to_string(), "Hannover Rück SE".to_string(), "XETR".to_string()),
        ("SHL".to_string(), "Siemens Healthineers AG".to_string(), "XETR".to_string()),
        ("PSM".to_string(), "ProSiebenSat.1 Media SE".to_string(), "XETR".to_string()),
        ("MTX".to_string(), "MTU Aero Engines AG".to_string(), "XETR".to_string()),
        ("SY1".to_string(), "Symrise AG".to_string(), "XETR".to_string()),
        ("PUM".to_string(), "Puma SE".to_string(), "XETR".to_string()),
        ("QIA".to_string(), "Qiagen N.V.".to_string(), "XETR".to_string()),
        ("DTG".to_string(), "Daimler Truck Holding AG".to_string(), "XETR".to_string()),
        ("PAH3".to_string(), "Porsche Automobil Holding SE".to_string(), "XETR".to_string()),
        ("MRK".to_string(), "Merck KGaA".to_string(), "XETR".to_string()),
        ("GXI".to_string(), "Gerresheimer AG".to_string(), "XETR".to_string()),
        ("SRT3".to_string(), "Sartorius AG".to_string(), "XETR".to_string()),
    ]
}

