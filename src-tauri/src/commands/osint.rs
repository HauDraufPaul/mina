use crate::storage::osint::OSINTStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;
use rusqlite::params;

#[tauri::command]
pub fn create_rss_feed(
    url: String,
    name: String,
    reliability: Option<f64>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_feed(&url, &name, reliability)
        .map_err(|e| format!("Failed to create feed: {}", e))
}

#[tauri::command]
pub fn update_rss_feed(
    id: i64,
    name: Option<String>,
    url: Option<String>,
    reliability: Option<f64>,
    enabled: Option<bool>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.update_feed(id, name.as_deref(), url.as_deref(), reliability, enabled)
        .map_err(|e| format!("Failed to update feed: {}", e))
}

#[tauri::command]
pub fn delete_rss_feed(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.delete_feed(id)
        .map_err(|e| format!("Failed to delete feed: {}", e))
}

#[tauri::command]
pub fn list_rss_feeds(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::RSSFeed>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.list_feeds()
        .map_err(|e| format!("Failed to list feeds: {}", e))
}

#[tauri::command]
pub fn save_rss_item(
    feed_id: i64,
    title: String,
    content: String,
    url: String,
    published_at: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.save_rss_item(feed_id, &title, &content, &url, published_at)
        .map_err(|e| format!("Failed to save RSS item: {}", e))
}

#[tauri::command]
pub fn get_recent_rss_items(
    limit: i32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::RSSItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.get_recent_items(limit)
        .map_err(|e| format!("Failed to get RSS items: {}", e))
}

#[tauri::command]
pub fn create_entity(
    entity_type: String,
    name: String,
    metadata: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_entity(&entity_type, &name, &metadata)
        .map_err(|e| format!("Failed to create entity: {}", e))
}

#[tauri::command]
pub fn list_entities(
    entity_type: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::Entity>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.list_entities(entity_type.as_deref())
        .map_err(|e| format!("Failed to list entities: {}", e))
}

#[tauri::command]
pub fn create_entity_relationship(
    source_id: i64,
    target_id: i64,
    relationship_type: String,
    strength: f64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_relationship(source_id, target_id, &relationship_type, strength)
        .map_err(|e| format!("Failed to create relationship: {}", e))
}

// Helper function to extract article content from HTML
async fn fetch_full_article_content(url: &str) -> Option<String> {
    use scraper::{Html, Selector};
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .ok()?;
    
    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(html) => {
                        let document = Html::parse_document(&html);
                        
                        // Try common article content selectors
                        let content_selectors = vec![
                            "article",
                            ".article-content",
                            ".post-content",
                            ".entry-content",
                            ".content",
                            "main article",
                            "[role='article']",
                            ".article-body",
                            ".story-body",
                            ".article-text",
                        ];
                        
                        for selector_str in content_selectors {
                            if let Ok(selector) = Selector::parse(selector_str) {
                                if let Some(element) = document.select(&selector).next() {
                                    let mut content = element.html();
                                    
                                    // Clean up the HTML
                                    content = content
                                        .replace("<script", "<!--script")
                                        .replace("</script>", "</script-->")
                                        .replace("<style", "<!--style")
                                        .replace("</style>", "</style-->");
                                    
                                    if content.len() > 500 {
                                        return Some(content);
                                    }
                                }
                            }
                        }
                        
                        // Fallback: try to get main content area
                        if let Ok(main_selector) = Selector::parse("main") {
                            if let Some(element) = document.select(&main_selector).next() {
                                let content = element.html();
                                if content.len() > 500 {
                                    return Some(content);
                                }
                            }
                        }
                        
                        // Last resort: get body content
                        if let Ok(body_selector) = Selector::parse("body") {
                            if let Some(element) = document.select(&body_selector).next() {
                                let content = element.html();
                                if content.len() > 500 {
                                    return Some(content);
                                }
                            }
                        }
                    }
                    Err(_) => return None,
                }
            }
        }
        Err(_) => return None,
    }
    
    None
}

#[tauri::command]
pub async fn fetch_rss_feeds(
    db: State<'_, Mutex<Database>>,
) -> Result<usize, String> {
    use crate::storage::osint::OSINTStore;
    use rss::Channel;
    use std::io::Cursor;
    
    // Get all enabled feeds first (release lock before async operations)
    let feeds = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let store = OSINTStore::new(db_guard.conn.clone());
        store.list_feeds()
            .map_err(|e| format!("Failed to list feeds: {}", e))?
    };
    
    let enabled_feeds: Vec<_> = feeds.into_iter().filter(|f| f.enabled).collect();
    
    if enabled_feeds.is_empty() {
        return Ok(0);
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let mut total_items = 0;
    let now = chrono::Utc::now().timestamp();
    
    // Collect all items first, then save them
    let mut items_to_save: Vec<(i64, String, String, String, i64)> = Vec::new();
    let mut feeds_to_update: Vec<i64> = Vec::new();
    
    for feed in enabled_feeds {
        match client.get(&feed.url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(content) => {
                            match Channel::read_from(Cursor::new(content.as_bytes())) {
                                Ok(channel) => {
                                    let mut items_saved = 0;
                                    for item in channel.items() {
                                        let title = item.title().unwrap_or("Untitled").to_string();
                                        let link = item.link().unwrap_or("").to_string();
                                        
                                        // Try to get full content from various RSS fields
                                        let mut content = String::new();
                                        
                                        // Try to get content from description (most RSS feeds put content here)
                                        let description = item.description().unwrap_or("").to_string();
                                        
                                        // Check if description contains HTML and is substantial (likely full content)
                                        let is_full_content = description.contains("<p>") || 
                                                             description.contains("<div>") || 
                                                             description.contains("<article>") ||
                                                             (description.len() > 1000 && description.contains("<"));
                                        
                                        if is_full_content {
                                            // This looks like full content
                                            content = description;
                                        } else if description.len() > 500 {
                                            // Medium length - might be full content in plain text
                                            content = format!("<p>{}</p>", description);
                                        } else {
                                            // Short description - try to fetch full article (but don't block if it fails)
                                            if !link.is_empty() && description.len() < 300 {
                                                // Only fetch if it's clearly a summary
                                                eprintln!("RSS item has short summary ({} chars), fetching full article from: {}", description.len(), link);
                                                if let Some(full_content) = fetch_full_article_content(&link).await {
                                                    content = full_content;
                                                    eprintln!("✓ Successfully fetched full article content ({} chars)", content.len());
                                                } else {
                                                    // Fallback to description with link
                                                    if !description.is_empty() {
                                                        content = format!("<p>{}</p>", description);
                                                    }
                                                    content.push_str(&format!("<p class=\"read-more\"><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"read-full-article\">📖 Read full article on original site →</a></p>", link));
                                                    eprintln!("✗ Failed to fetch full article, using summary");
                                                }
                                            } else {
                                                // Use description as-is
                                                if !description.is_empty() {
                                                    content = format!("<p>{}</p>", description);
                                                }
                                            }
                                        }
                                        
                                        // If content is still empty, provide a fallback
                                        if content.trim().is_empty() {
                                            content = format!("<p class=\"no-content\">No content preview available.</p><p class=\"read-more\"><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"read-full-article\">📖 Read full article on original site →</a></p>", link);
                                        }
                                        
                                        // Parse published date
                                        let published_at = item.pub_date()
                                            .and_then(|date| {
                                                chrono::DateTime::parse_from_rfc2822(date)
                                                    .or_else(|_| chrono::DateTime::parse_from_rfc3339(date))
                                                    .ok()
                                            })
                                            .map(|dt| dt.timestamp())
                                            .unwrap_or(now);
                                        
                                        items_to_save.push((feed.id, title, content, link, published_at));
                                        items_saved += 1;
                                    }
                                    
                                    if items_saved > 0 {
                                        feeds_to_update.push(feed.id);
                                        total_items += items_saved;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse RSS feed {}: {}", feed.name, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read RSS feed {}: {}", feed.name, e);
                        }
                    }
                } else {
                    eprintln!("Failed to fetch RSS feed {}: HTTP {}", feed.name, response.status());
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch RSS feed {}: {}", feed.name, e);
            }
        }
    }
    
    // Now save all items to database (with lock)
    {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let store = OSINTStore::new(db_guard.conn.clone());
        
        for (feed_id, title, description, link, published_at) in items_to_save {
            let _ = store.save_rss_item(feed_id, &title, &description, &link, published_at);
        }
        
        for feed_id in feeds_to_update {
            let _ = store.update_feed_last_fetch(feed_id);
        }
    }
    
    Ok(total_items)
}

#[tauri::command]
pub fn get_rss_item(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::osint::RSSItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.get_item(id)
        .map_err(|e| format!("Failed to get RSS item: {}", e))
}

#[tauri::command]
pub fn mark_article_read(
    id: i64,
    read: bool,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.mark_as_read(id, read)
        .map_err(|e| format!("Failed to mark article: {}", e))
}

#[tauri::command]
pub fn toggle_article_favorite(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.toggle_favorite(id)
        .map_err(|e| format!("Failed to toggle favorite: {}", e))
}

#[tauri::command]
pub fn toggle_article_saved(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.toggle_saved(id)
        .map_err(|e| format!("Failed to toggle saved: {}", e))
}

#[tauri::command]
pub fn set_article_folder(
    id: i64,
    folder_id: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.set_folder(id, folder_id)
        .map_err(|e| format!("Failed to set folder: {}", e))
}

#[tauri::command]
pub fn create_article_folder(
    name: String,
    color: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_folder(&name, color.as_deref())
        .map_err(|e| format!("Failed to create folder: {}", e))
}

#[tauri::command]
pub fn list_article_folders(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::ArticleFolder>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.list_folders()
        .map_err(|e| format!("Failed to list folders: {}", e))
}

#[tauri::command]
pub fn delete_article_folder(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.delete_folder(id)
        .map_err(|e| format!("Failed to delete folder: {}", e))
}

#[tauri::command]
pub fn get_filtered_articles(
    favorite: Option<bool>,
    saved: Option<bool>,
    read: Option<bool>,
    folder_id: Option<i64>,
    limit: i32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::RSSItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.get_items_by_filter(favorite, saved, read, folder_id, limit)
        .map_err(|e| format!("Failed to get filtered articles: {}", e))
}

#[tauri::command]
pub fn get_article_entities(
    article_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::ExtractedEntity>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.get_entities_for_article(article_id)
        .map_err(|e| format!("Failed to get entities: {}", e))
}

#[tauri::command]
pub async fn extract_entities_from_article(
    article_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<usize, String> {
    use crate::storage::osint::OSINTStore;
    
    // Get article content
    let (title, content) = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let store = OSINTStore::new(db_guard.conn.clone());
        match store.get_item(article_id) {
            Ok(Some(item)) => (item.title, item.content),
            Ok(None) => return Err("Article not found".to_string()),
            Err(e) => return Err(format!("Failed to get article: {}", e)),
        }
    };

    // Enhanced entity extraction
    let text = format!("{} {}", title, content);
    let entities = extract_entities_enhanced(&text);

    // Save entities
    {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let store = OSINTStore::new(db_guard.conn.clone());
        let mut saved = 0;
        for (entity_type, name, confidence, context) in entities {
            if let Ok(_) = store.save_extracted_entity(article_id, &entity_type, &name, confidence, Some(&context)) {
                saved += 1;
            }
        }
        Ok(saved)
    }
}

#[tauri::command]
pub async fn fetch_full_article(
    article_id: i64,
    url: String,
    db: State<'_, Mutex<Database>>,
) -> Result<String, String> {
    use crate::storage::osint::OSINTStore;
    
    // Fetch full article content
    if let Some(full_content) = fetch_full_article_content(&url).await {
        // Update the article in database
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        let store = OSINTStore::new(db_guard.conn.clone());
        
        // Verify article exists
        match store.get_item(article_id) {
            Ok(Some(_)) => {
                // Update content using the store's connection
                let conn = store.conn.lock().map_err(|e| format!("Database lock error: {}", e))?;
                conn.execute(
                    "UPDATE rss_items SET content = ?1 WHERE id = ?2",
                    params![full_content, article_id],
                ).map_err(|e| format!("Failed to update article: {}", e))?;
                
                Ok("Full article content fetched successfully".to_string())
            }
            Ok(None) => Err("Article not found".to_string()),
            Err(e) => Err(format!("Failed to get article: {}", e)),
        }
    } else {
        Err("Failed to fetch full article content".to_string())
    }
}

fn extract_entities_enhanced(text: &str) -> Vec<(String, String, f64, String)> {
    let mut entities = Vec::new();
    let lower = text.to_lowercase();
    let words: Vec<&str> = text.split_whitespace().collect();

    // Enhanced country patterns (comprehensive list)
    let countries = vec![
        "united states", "usa", "us", "america", "united states of america",
        "china", "peoples republic of china", "prc",
        "russia", "russian federation",
        "japan",
        "germany", "federal republic of germany",
        "france", "french republic",
        "united kingdom", "uk", "britain", "great britain", "england", "scotland", "wales",
        "india", "republic of india",
        "brazil", "federative republic of brazil",
        "canada",
        "australia", "commonwealth of australia",
        "south korea", "republic of korea", "rok",
        "north korea", "dprk", "democratic peoples republic of korea",
        "italy", "italian republic",
        "spain", "kingdom of spain",
        "netherlands", "holland",
        "sweden", "kingdom of sweden",
        "norway", "kingdom of norway",
        "denmark", "kingdom of denmark",
        "finland", "republic of finland",
        "poland", "republic of poland",
        "belgium", "kingdom of belgium",
        "switzerland", "swiss confederation",
        "austria", "republic of austria",
        "israel", "state of israel",
        "singapore", "republic of singapore",
        "taiwan", "republic of china", "roc",
        "south africa", "republic of south africa",
        "mexico", "united mexican states",
        "argentina", "argentine republic",
        "chile", "republic of chile",
        "egypt", "arab republic of egypt",
        "turkey", "republic of turkey", "turkiye",
        "thailand", "kingdom of thailand",
        "indonesia", "republic of indonesia",
        "philippines", "republic of the philippines",
        "vietnam", "socialist republic of vietnam",
        "malaysia",
        "new zealand",
        "ireland", "republic of ireland",
        "portugal", "portuguese republic",
        "greece", "hellenic republic",
        "czech republic", "czechia",
        "romania",
        "hungary",
        "ukraine",
        "pakistan", "islamic republic of pakistan",
        "bangladesh", "peoples republic of bangladesh",
        "iran", "islamic republic of iran",
        "iraq", "republic of iraq",
        "saudi arabia", "kingdom of saudi arabia",
        "uae", "united arab emirates",
    ];
    
    for country in countries {
        if lower.contains(country) {
            let display_name = match country {
                "usa" | "us" => "United States",
                "uk" => "United Kingdom",
                "uae" => "United Arab Emirates",
                "dprk" => "North Korea",
                "rok" => "South Korea",
                "prc" => "China",
                "roc" => "Taiwan",
                _ => country,
            };
            entities.push(("country".to_string(), display_name.to_string(), 0.8, text.to_string()));
        }
    }

    // Enhanced company patterns (comprehensive tech and major companies)
    let companies = vec![
        "apple", "apple inc", "apple computer",
        "google", "alphabet", "alphabet inc",
        "microsoft", "microsoft corporation",
        "amazon", "amazon.com", "amazon web services", "aws",
        "meta", "facebook", "meta platforms",
        "tesla", "tesla motors", "tesla inc",
        "nvidia", "nvidia corporation",
        "intel", "intel corporation",
        "amd", "advanced micro devices",
        "samsung", "samsung electronics",
        "ibm", "international business machines",
        "oracle", "oracle corporation",
        "salesforce", "salesforce.com",
        "netflix", "netflix inc",
        "twitter", "x.com", "x corp",
        "openai",
        "anthropic",
        "github",
        "linkedin", "linkedin corporation",
        "uber", "uber technologies",
        "airbnb", "airbnb inc",
        "spotify",
        "adobe", "adobe systems",
        "cisco", "cisco systems",
        "qualcomm", "qualcomm incorporated",
        "broadcom",
        "paypal",
        "visa", "visa inc",
        "mastercard",
        "goldman sachs",
        "jpmorgan", "jpmorgan chase",
        "morgan stanley",
        "blackrock",
        "vanguard",
        "disney", "walt disney",
        "sony", "sony corporation",
        "panasonic",
        "lg", "lg electronics",
        "huawei",
        "xiaomi",
        "tencent",
        "alibaba", "alibaba group",
        "bytedance", "tiktok",
        "zoom", "zoom video communications",
        "slack", "slack technologies",
        "dropbox",
        "atlassian",
        "shopify",
        "square", "block inc",
        "coinbase",
        "binance",
    ];
    
    for company in companies {
        if lower.contains(company) {
            let display_name = company.split_whitespace().map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }).collect::<Vec<_>>().join(" ");
            entities.push(("company".to_string(), display_name, 0.85, text.to_string()));
        }
    }

    // Enhanced technology keywords
    let tech_patterns = vec![
        ("ai", "Artificial Intelligence"),
        ("artificial intelligence", "Artificial Intelligence"),
        ("machine learning", "Machine Learning"),
        ("ml", "Machine Learning"),
        ("deep learning", "Deep Learning"),
        ("neural network", "Neural Network"),
        ("blockchain", "Blockchain"),
        ("cryptocurrency", "Cryptocurrency"),
        ("crypto", "Cryptocurrency"),
        ("bitcoin", "Bitcoin"),
        ("ethereum", "Ethereum"),
        ("quantum computing", "Quantum Computing"),
        ("cloud computing", "Cloud Computing"),
        ("iot", "Internet of Things"),
        ("internet of things", "Internet of Things"),
        ("5g", "5G Network"),
        ("6g", "6G Network"),
        ("vr", "Virtual Reality"),
        ("virtual reality", "Virtual Reality"),
        ("ar", "Augmented Reality"),
        ("augmented reality", "Augmented Reality"),
        ("metaverse", "Metaverse"),
        ("cybersecurity", "Cybersecurity"),
        ("ransomware", "Ransomware"),
        ("malware", "Malware"),
        ("phishing", "Phishing"),
        ("api", "API"),
        ("saas", "SaaS"),
        ("paas", "PaaS"),
        ("iaas", "IaaS"),
    ];
    
    for (pattern, display) in tech_patterns {
        if lower.contains(pattern) {
            entities.push(("technology".to_string(), display.to_string(), 0.7, text.to_string()));
        }
    }

    // Enhanced person name extraction (better pattern matching)
    for i in 0..words.len().saturating_sub(1) {
        let word1 = words[i].trim_matches(|c: char| !c.is_alphanumeric());
        let word2 = words[i + 1].trim_matches(|c: char| !c.is_alphanumeric());
        
        if word1.len() >= 2 && word2.len() >= 2 &&
           word1.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) &&
           word2.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) &&
           word1.chars().all(|c| c.is_alphabetic()) &&
           word2.chars().all(|c| c.is_alphabetic()) {
            
            // Filter out common false positives
            let common_words = vec!["the", "and", "for", "are", "but", "not", "you", "all", "can", "her", "was", "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new", "now", "old", "see", "two", "way", "who", "boy", "did", "its", "let", "put", "say", "she", "too", "use"];
            let word1_lower = word1.to_lowercase();
            let word2_lower = word2.to_lowercase();
            
            if !common_words.contains(&word1_lower.as_str()) &&
               !common_words.contains(&word2_lower.as_str()) &&
               word1_lower != "mr" && word1_lower != "mrs" && word1_lower != "ms" &&
               word1_lower != "dr" && word1_lower != "prof" {
                let name = format!("{} {}", word1, word2);
                if !entities.iter().any(|(_, n, _, _)| n.eq_ignore_ascii_case(&name)) {
                    entities.push(("person".to_string(), name, 0.6, text.to_string()));
                }
            }
        }
    }

    // Extract URLs as connections
    let url_start = "http://";
    let url_start_https = "https://";
    let mut start = 0;
    while let Some(pos) = text[start..].find(url_start).or_else(|| text[start..].find(url_start_https)) {
        let actual_pos = start + pos;
        let remaining = &text[actual_pos..];
        if let Some(end) = remaining.find(char::is_whitespace) {
            let url = remaining[..end].to_string();
            if url.len() < 200 && url.contains('.') {
                entities.push(("connection".to_string(), url, 0.95, text.to_string()));
            }
            start = actual_pos + end;
        } else {
            let url = remaining.to_string();
            if url.len() < 200 && url.contains('.') {
                entities.push(("connection".to_string(), url, 0.95, text.to_string()));
            }
            break;
        }
    }

    // Extract email addresses
    let email_pattern = "@";
    let mut start = 0;
    while let Some(pos) = text[start..].find(email_pattern) {
        let actual_pos = start + pos;
        let before = &text[..actual_pos];
        let after = &text[actual_pos + 1..];
        if let Some(email_start) = before.rfind(char::is_whitespace) {
            if let Some(email_end) = after.find(char::is_whitespace) {
                let email = text[email_start + 1..actual_pos + 1 + email_end].to_string();
                if email.contains('@') && email.contains('.') && email.len() < 100 {
                    entities.push(("connection".to_string(), email, 0.9, text.to_string()));
                }
                start = actual_pos + 1 + email_end;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Extract IP addresses
    let ip_patterns = vec!["192.168.", "10.0.", "172.16.", "127.0.0.1"];
    for pattern in ip_patterns {
        if lower.contains(pattern) {
            // Try to extract full IP
            if let Some(pos) = text.find(pattern) {
                let start_pos = pos.saturating_sub(0);
                let remaining = &text[start_pos..];
                let parts: Vec<&str> = remaining.split_whitespace().collect();
                if let Some(first_part) = parts.first() {
                    if first_part.matches('.').count() >= 2 {
                        entities.push(("connection".to_string(), first_part.to_string(), 0.85, text.to_string()));
                    }
                }
            }
        }
    }

    // Extract dates and events
    let date_patterns = vec![
        "january", "february", "march", "april", "may", "june",
        "july", "august", "september", "october", "november", "december",
        "2024", "2025", "2026", "2023", "2022",
    ];
    for pattern in date_patterns {
        if lower.contains(pattern) {
            entities.push(("event".to_string(), pattern.to_string(), 0.5, text.to_string()));
        }
    }

    // Remove duplicates and sort by confidence
    entities.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    entities.dedup_by(|a, b| a.0 == b.0 && a.1.eq_ignore_ascii_case(&b.1));
    
    entities
}
