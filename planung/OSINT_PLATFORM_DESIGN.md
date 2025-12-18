# OSIRIS - Privacy-First Unified OSINT Platform
## A Graph-Based Approach to Open Source Intelligence

### 🎓 Research Thesis
**"Privacy-First OSINT: A Unified Graph-Based Architecture for Open Source Intelligence Collection and Analysis"**

---

## 🧠 Core Research Question

**How can we build a unified OSINT platform that integrates heterogeneous data sources into a single graph model while maintaining privacy, enabling real-time monitoring, and automatically detecting relationships across sources?**

Traditional OSINT tools are fragmented, expensive, and privacy-invasive. We propose a local-first, graph-based platform that unifies all intelligence sources.

---

## 🏗️ Architecture: Unified Intelligence Graph

### Core Innovation: Cross-Source Relationship Graph

Instead of separate tools for separate sources, we model:
```
Source → Entity Extraction → Relationship Detection → Unified Graph → Query/Visualize
```

### Technology Stack (No AI Required)

#### **Frontend: Rust + Tauri + Leptos**
- **Why Rust**: Performance, memory safety, perfect for data processing
- **Leptos**: Modern Rust web framework, reactive, fast
- **Tauri**: Native desktop with Rust backend
- **No JavaScript**: Everything in Rust for performance

#### **Backend: Rust + Tokio**
- **Async Runtime**: Tokio for high-performance async I/O
- **HTTP Client**: `reqwest` for API calls, `scraper` for web scraping
- **Rate Limiting**: Built-in rate limiting per source
- **No external AI services**: Pure algorithmic approaches

#### **Database: SQLite + Neo4j (Optional)**
- **SQLite**: Primary storage (local, fast, reliable)
- **Neo4j**: Optional for advanced graph queries (if user wants)
- **Full-text Search**: SQLite FTS5 for fast text search
- **Graph Operations**: Custom graph algorithms in Rust

#### **Query Language: OSINT Query Language (OQL)**
```rust
// Example: Find all entities related to a domain
query {
    find entity: Domain
    where domain = "example.com"
    expand relationships: all
    depth: 3
    sources: [WHOIS, DNS, Twitter, LinkedIn]
    confidence: > 0.7
}
```

---

## 🔬 Novel Research Contributions

### 1. **Cross-Source Relationship Detection**

**Problem**: Data from different sources doesn't connect automatically.

**Solution**: Rule-based relationship detection algorithms.

```rust
struct RelationshipRule {
    name: String,
    source_types: Vec<EntityType>,
    matching_criteria: Vec<MatchingCriterion>,
    confidence_calculator: ConfidenceCalculator,
}

// Example: Email domain matching
RelationshipRule {
    name: "email_domain_match",
    source_types: vec![EntityType::Email, EntityType::Domain],
    matching_criteria: vec![
        MatchingCriterion::EmailDomainMatchesDomain,
    ],
    confidence_calculator: ConfidenceCalculator::High(0.9),
}

// Example: Username pattern matching
RelationshipRule {
    name: "username_pattern_match",
    source_types: vec![EntityType::Twitter, EntityType::GitHub, EntityType::LinkedIn],
    matching_criteria: vec![
        MatchingCriterion::UsernameSimilarity(0.8),
        MatchingCriterion::NameMatches,
    ],
    confidence_calculator: ConfidenceCalculator::Medium(0.7),
}
```

**Algorithm**:
```rust
fn detect_relationships(entity: &Entity, graph: &Graph) -> Vec<Relationship> {
    let mut relationships = Vec::new();
    
    // Apply all matching rules
    for rule in &RELATIONSHIP_RULES {
        if rule.applies_to(entity.entity_type) {
            let matches = rule.find_matches(entity, graph);
            for (matched_entity, confidence) in matches {
                relationships.push(Relationship {
                    from: entity.id,
                    to: matched_entity.id,
                    relationship_type: rule.name.clone(),
                    confidence,
                    evidence: rule.collect_evidence(entity, &matched_entity),
                });
            }
        }
    }
    
    relationships
}
```

### 2. **Confidence Scoring System**

**Problem**: How do we know if data is reliable?

**Solution**: Multi-factor confidence scoring.

```rust
struct ConfidenceScore {
    source_reliability: f64,      // How reliable is the source?
    data_freshness: f64,           // How recent is the data?
    cross_source_verification: f64, // Verified by multiple sources?
    pattern_match_quality: f64,    // How well does it match patterns?
    overall: f64,                  // Combined score
}

fn calculate_confidence(entity: &Entity, graph: &Graph) -> ConfidenceScore {
    let source_reliability = get_source_reliability(&entity.source);
    let data_freshness = calculate_freshness(&entity.last_seen);
    let cross_source = count_verifying_sources(entity, graph);
    let pattern_quality = match_known_patterns(entity);
    
    let overall = (
        source_reliability * 0.3 +
        data_freshness * 0.2 +
        cross_source * 0.3 +
        pattern_quality * 0.2
    );
    
    ConfidenceScore { source_reliability, data_freshness, cross_source, pattern_quality, overall }
}
```

### 3. **Real-Time Monitoring Architecture**

**Problem**: Batch queries miss time-sensitive information.

**Solution**: Continuous monitoring with efficient polling and change detection.

```rust
struct Monitor {
    id: MonitorId,
    source: Source,
    query: Query,
    poll_interval: Duration,
    alert_conditions: Vec<AlertCondition>,
    last_check: Timestamp,
    last_state: MonitorState,
}

struct AlertCondition {
    condition: Condition, // "new entity", "entity changed", "pattern matched"
    action: AlertAction,  // Notify, collect, analyze
}

// Efficient change detection
fn check_monitor(monitor: &mut Monitor) -> Result<Vec<Alert>> {
    let current_state = monitor.source.query(&monitor.query)?;
    let changes = detect_changes(&monitor.last_state, &current_state);
    
    let mut alerts = Vec::new();
    for change in changes {
        for condition in &monitor.alert_conditions {
            if condition.matches(&change) {
                alerts.push(Alert {
                    monitor_id: monitor.id,
                    change,
                    action: condition.action.clone(),
                });
            }
        }
    }
    
    monitor.last_state = current_state;
    monitor.last_check = now();
    Ok(alerts)
}
```

### 4. **Source Abstraction Layer**

**Problem**: Each source has different API, format, rate limits.

**Solution**: Unified source interface.

```rust
trait Source: Send + Sync {
    fn name(&self) -> &str;
    fn collect(&self, query: &Query) -> Result<Vec<Entity>>;
    fn monitor(&self, query: &Query) -> Result<Monitor>;
    fn rate_limit(&self) -> RateLimit;
    fn requires_auth(&self) -> bool;
    fn authenticate(&mut self, credentials: Credentials) -> Result<()>;
}

// Example: RSS Source
struct RSSSource {
    url: String,
    last_fetch: Option<Timestamp>,
}

impl Source for RSSSource {
    fn collect(&self, query: &Query) -> Result<Vec<Entity>> {
        let feed = fetch_rss(&self.url)?;
        let entities = extract_entities_from_feed(&feed, query)?;
        Ok(entities)
    }
    
    fn monitor(&self, query: &Query) -> Result<Monitor> {
        Ok(Monitor {
            source: SourceType::RSS,
            query: query.clone(),
            poll_interval: Duration::from_secs(300), // 5 minutes
            // ...
        })
    }
    
    fn rate_limit(&self) -> RateLimit {
        RateLimit::per_minute(10) // RSS is usually generous
    }
}

// Example: Twitter Source (using API)
struct TwitterSource {
    client: TwitterClient,
    rate_limiter: RateLimiter,
}

impl Source for TwitterSource {
    fn collect(&self, query: &Query) -> Result<Vec<Entity>> {
        self.rate_limiter.wait_if_needed()?;
        let tweets = self.client.search(&query.to_twitter_query())?;
        let entities = extract_entities_from_tweets(&tweets)?;
        Ok(entities)
    }
    
    fn rate_limit(&self) -> RateLimit {
        RateLimit::per_15min(180) // Twitter API limits
    }
}
```

### 5. **Pattern Matching System**

**Problem**: Need to detect patterns in data (not AI, just rules).

**Solution**: Rule-based pattern matching.

```rust
struct Pattern {
    id: PatternId,
    name: String,
    description: String,
    rules: Vec<PatternRule>,
    confidence: f64,
}

struct PatternRule {
    condition: Condition,
    weight: f64,
}

// Example: "New domain registration pattern"
Pattern {
    id: "new_domain_pattern",
    name: "New Domain Registration",
    rules: vec![
        PatternRule {
            condition: Condition::EntityType(EntityType::Domain),
            weight: 1.0,
        },
        PatternRule {
            condition: Condition::Age(Duration::from_days(7)),
            weight: 0.8,
        },
        PatternRule {
            condition: Condition::HasRelationship("registered_by"),
            weight: 0.9,
        },
    ],
    confidence: 0.85,
}

fn match_pattern(entity: &Entity, pattern: &Pattern) -> Option<f64> {
    let mut score = 0.0;
    let mut total_weight = 0.0;
    
    for rule in &pattern.rules {
        if rule.condition.matches(entity) {
            score += rule.weight;
        }
        total_weight += rule.weight;
    }
    
    if total_weight > 0.0 {
        Some(score / total_weight)
    } else {
        None
    }
}
```

### 6. **Privacy-First Architecture**

**Problem**: Cloud-based tools store your data.

**Solution**: Local-first with optional encrypted sync.

```rust
struct Storage {
    local_db: Arc<Database>,           // SQLite local database
    encrypted_sync: Option<EncryptedSync>, // Optional cloud sync
    encryption_key: Key,               // User-controlled encryption
}

struct EncryptedSync {
    server_url: String,
    encrypted_data: Vec<u8>,          // Data encrypted before upload
    sync_enabled: bool,
}

// All data encrypted before leaving local machine
fn sync_to_cloud(storage: &Storage, data: &[Entity]) -> Result<()> {
    if let Some(sync) = &storage.encrypted_sync {
        if sync.sync_enabled {
            let encrypted = encrypt(data, &storage.encryption_key)?;
            upload_encrypted(sync.server_url, encrypted)?;
        }
    }
    Ok(())
}

// Local-first: Everything works offline
fn query_local(storage: &Storage, query: &Query) -> Result<Vec<Entity>> {
    storage.local_db.query(query) // Always works, no network needed
}
```

---

## 🎯 Core Features (No AI)

### 1. **Unified Intelligence Graph**

Visualize all intelligence in one graph:
- **Entity Types**: Person, Domain, IP, Email, Twitter, LinkedIn, etc.
- **Relationships**: Automatic detection across sources
- **Confidence Scores**: Visual indicators of data reliability
- **Source Attribution**: See where each piece of data came from

### 2. **Real-Time Monitoring**

Monitor sources continuously:
- **RSS Feeds**: Check for new articles matching queries
- **Domain Registrations**: Alert on new domains
- **Social Media**: Monitor for mentions, new accounts
- **Custom Queries**: Monitor any queryable source

### 3. **Cross-Source Relationship Detection**

Automatically find connections:
- **Email-Domain**: Email uses domain → link to domain entity
- **Username Matching**: Same username across platforms
- **IP Associations**: IPs linked to domains, services
- **Name Variations**: "John Smith" = "J. Smith" = "Johnny Smith"

### 4. **Query Interface**

Powerful query language:
```rust
// Find all people related to a domain
query {
    find: Person
    related_to: {
        Domain: "example.com"
        relationship: any
    }
    sources: [Twitter, LinkedIn, WHOIS]
    confidence: > 0.7
}

// Find all domains registered in last 7 days
query {
    find: Domain
    where: {
        registered_date > now() - 7 days
    }
    expand: {
        relationships: all
        depth: 2
    }
}
```

### 5. **Source Management**

Add and configure sources:
- **Built-in Sources**: RSS, WHOIS, DNS, Twitter, LinkedIn, GitHub, etc.
- **Custom Sources**: Plugin system for new sources
- **Authentication**: Store API keys securely
- **Rate Limiting**: Automatic rate limit management

### 6. **Investigation Management**

Organize investigations:
- **Projects**: Group related queries and entities
- **Timeline View**: See how investigation evolved
- **Export**: Export investigations (JSON, CSV, GraphML)
- **Collaboration**: Share investigations (encrypted)

### 7. **Visualization**

Multiple visualization modes:
- **Graph View**: Interactive graph of entities and relationships
- **Timeline View**: See entities over time
- **Table View**: Traditional table with filtering
- **Map View**: Geographic visualization (if location data)

---

## 🛠️ Implementation Details

### Core Data Structures

```rust
// Entity: Core data structure
#[derive(Clone, Debug)]
struct Entity {
    id: EntityId,
    entity_type: EntityType,
    attributes: HashMap<String, Value>,
    sources: Vec<SourceInfo>,
    relationships: Vec<RelationshipId>,
    confidence: f64,
    created_at: Timestamp,
    updated_at: Timestamp,
    last_seen: Timestamp,
}

#[derive(Clone, Debug)]
enum EntityType {
    Person { name: String, emails: Vec<String> },
    Domain { domain: String, registrar: Option<String> },
    IP { ip: IpAddr, asn: Option<u32> },
    Email { email: String, domain: String },
    Twitter { username: String, user_id: Option<String> },
    LinkedIn { profile_url: String },
    GitHub { username: String },
    Article { url: String, title: String, published: Timestamp },
    // ... more types
}

// Relationship: Connection between entities
#[derive(Clone, Debug)]
struct Relationship {
    id: RelationshipId,
    from: EntityId,
    to: EntityId,
    relationship_type: RelationshipType,
    confidence: f64,
    evidence: Vec<Evidence>,
    created_at: Timestamp,
}

#[derive(Clone, Debug)]
enum RelationshipType {
    UsesEmailDomain,
    HasTwitterAccount,
    RegisteredDomain,
    MentionsInArticle,
    SameIP,
    UsernameMatch,
    NameMatch,
    // ... more types
}

// Evidence: Why we think this relationship exists
#[derive(Clone, Debug)]
struct Evidence {
    source: SourceInfo,
    data: String,
    confidence: f64,
}
```

### Key Algorithms

#### 1. Relationship Detection Algorithm
```rust
fn detect_relationships(entity: &Entity, graph: &Graph) -> Vec<Relationship> {
    let mut relationships = Vec::new();
    
    // Email domain matching
    if let EntityType::Email { email, domain } = &entity.entity_type {
        if let Some(domain_entity) = graph.find_entity_by_domain(domain) {
            relationships.push(Relationship {
                from: entity.id,
                to: domain_entity.id,
                relationship_type: RelationshipType::UsesEmailDomain,
                confidence: 0.95, // Very high confidence
                evidence: vec![Evidence {
                    source: entity.sources[0].clone(),
                    data: format!("Email {} uses domain {}", email, domain),
                    confidence: 0.95,
                }],
                created_at: now(),
            });
        }
    }
    
    // Username matching across platforms
    if let Some(username) = extract_username(entity) {
        let matches = graph.find_entities_by_username(&username);
        for matched in matches {
            if matched.id != entity.id {
                let confidence = calculate_username_match_confidence(entity, &matched);
                relationships.push(Relationship {
                    from: entity.id,
                    to: matched.id,
                    relationship_type: RelationshipType::UsernameMatch,
                    confidence,
                    evidence: vec![
                        Evidence {
                            source: entity.sources[0].clone(),
                            data: format!("Username match: {}", username),
                            confidence,
                        }
                    ],
                    created_at: now(),
                });
            }
        }
    }
    
    // Name matching (fuzzy)
    if let Some(name) = extract_name(entity) {
        let matches = graph.find_entities_by_name_fuzzy(&name, 0.8);
        for matched in matches {
            if matched.id != entity.id {
                let confidence = calculate_name_match_confidence(&name, &matched);
                if confidence > 0.7 {
                    relationships.push(Relationship {
                        from: entity.id,
                        to: matched.id,
                        relationship_type: RelationshipType::NameMatch,
                        confidence,
                        evidence: vec![Evidence {
                            source: entity.sources[0].clone(),
                            data: format!("Name match: {} ≈ {}", name, extract_name(&matched).unwrap()),
                            confidence,
                        }],
                        created_at: now(),
                    });
                }
            }
        }
    }
    
    relationships
}
```

#### 2. Confidence Calculation
```rust
fn calculate_confidence(entity: &Entity, graph: &Graph) -> f64 {
    let mut factors = Vec::new();
    
    // Source reliability
    let source_reliability = entity.sources.iter()
        .map(|s| get_source_reliability_score(s))
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.5);
    factors.push(("source", source_reliability, 0.3));
    
    // Data freshness
    let age = now() - entity.last_seen;
    let freshness = if age < Duration::from_days(1) {
        1.0
    } else if age < Duration::from_days(7) {
        0.8
    } else if age < Duration::from_days(30) {
        0.6
    } else {
        0.4
    };
    factors.push(("freshness", freshness, 0.2));
    
    // Cross-source verification
    let verifying_sources = count_verifying_sources(entity, graph);
    let verification_score = if verifying_sources >= 3 {
        1.0
    } else if verifying_sources == 2 {
        0.8
    } else if verifying_sources == 1 {
        0.5
    } else {
        0.3
    };
    factors.push(("verification", verification_score, 0.3));
    
    // Pattern matching
    let pattern_score = match_known_patterns(entity, graph);
    factors.push(("pattern", pattern_score, 0.2));
    
    // Weighted average
    let total_weight: f64 = factors.iter().map(|(_, _, w)| w).sum();
    let weighted_sum: f64 = factors.iter()
        .map(|(_, score, weight)| score * weight)
        .sum();
    
    weighted_sum / total_weight
}
```

#### 3. Pattern Matching
```rust
fn match_known_patterns(entity: &Entity, graph: &Graph) -> f64 {
    let patterns = get_pattern_library();
    let mut best_match = 0.0;
    
    for pattern in patterns {
        if let Some(score) = pattern.matches(entity) {
            if score > best_match {
                best_match = score;
            }
        }
    }
    
    best_match
}

// Example patterns
fn get_pattern_library() -> Vec<Pattern> {
    vec![
        Pattern {
            id: "new_domain",
            name: "New Domain Registration",
            rules: vec![
                PatternRule {
                    condition: Condition::EntityType(EntityType::Domain),
                    weight: 1.0,
                },
                PatternRule {
                    condition: Condition::Age(Duration::from_days(7)),
                    weight: 0.8,
                },
            ],
            confidence: 0.85,
        },
        Pattern {
            id: "suspicious_email",
            name: "Suspicious Email Pattern",
            rules: vec![
                PatternRule {
                    condition: Condition::EntityType(EntityType::Email),
                    weight: 1.0,
                },
                PatternRule {
                    condition: Condition::AttributeMatches("domain", |d| {
                        d.contains("temp") || d.contains("throwaway")
                    }),
                    weight: 0.9,
                },
            ],
            confidence: 0.7,
        },
        // ... more patterns
    ]
}
```

---

## 📊 User Interface Design

### Main Views

1. **Graph View**
   - Interactive graph visualization
   - Zoom, pan, filter
   - Click entities to see details
   - Highlight relationships

2. **Query Interface**
   - Natural language-like query builder
   - Query history
   - Save queries
   - Query templates

3. **Source Management**
   - Add/remove sources
   - Configure authentication
   - Test connections
   - View rate limits

4. **Monitoring Dashboard**
   - Active monitors
   - Recent alerts
   - Monitor statistics
   - Create new monitors

5. **Investigation Manager**
   - List of investigations
   - Timeline view
   - Export options
   - Collaboration settings

---

## 🔒 Privacy & Security

### Local-First Architecture
- All data stored locally by default
- No cloud required for core functionality
- User controls all data

### Optional Encrypted Sync
- End-to-end encryption
- User controls encryption keys
- Server cannot read data
- Optional feature, not required

### Data Minimization
- Only collect what's needed
- User controls what to collect
- Can delete data anytime
- No tracking, no analytics

---

## 🚀 Implementation Roadmap

### Phase 1: Core Foundation (Months 1-3)
- Basic entity model
- SQLite database
- Simple graph structure
- Basic UI (graph view)
- RSS source implementation

### Phase 2: Sources & Relationships (Months 4-6)
- Add more sources (WHOIS, DNS, Twitter, LinkedIn)
- Relationship detection algorithms
- Confidence scoring
- Pattern matching system

### Phase 3: Real-Time & Monitoring (Months 7-9)
- Real-time monitoring system
- Alert system
- Query language implementation
- Advanced visualization

### Phase 4: Polish & Advanced Features (Months 10-12)
- Investigation management
- Export/collaboration
- Performance optimization
- Documentation
- Plugin system

---

## 🎓 Research Contributions

### 1. **Cross-Source Relationship Detection Algorithm**
- Novel algorithm for detecting relationships across heterogeneous sources
- Confidence scoring methodology
- False positive reduction techniques

### 2. **Privacy-First OSINT Architecture**
- Local-first design patterns
- Encrypted collaboration mechanisms
- Data minimization strategies

### 3. **Real-Time OSINT Monitoring**
- Efficient polling strategies
- Change detection algorithms
- Scalable monitoring architecture

### 4. **Source Abstraction Framework**
- Unified interface for diverse sources
- Rate limiting strategies
- Error handling and retry logic

---

## 📈 Evaluation Metrics

### Performance
- Query response time (< 100ms for simple queries)
- Relationship detection speed
- Memory usage
- Database size

### Accuracy
- Relationship detection accuracy
- Confidence score calibration
- False positive rate
- Pattern matching accuracy

### Usability
- Time to first result
- Query complexity
- Learning curve
- User satisfaction

---

## 🎯 Why This Is Novel

1. **Unified Graph Model**: First OSINT platform with true cross-source graph
2. **Privacy-First**: Local-first architecture, no cloud required
3. **Real-Time**: Built-in monitoring, not just batch queries
4. **Open Source**: Free and accessible to everyone
5. **No AI Required**: Pure algorithmic approaches, explainable results

---

## 📚 Potential Publications

1. **"Privacy-First OSINT: A Local-First Architecture for Open Source Intelligence"**
   - Architecture design
   - Privacy guarantees
   - Performance evaluation

2. **"Cross-Source Relationship Detection in Heterogeneous OSINT Data"**
   - Relationship detection algorithms
   - Confidence scoring
   - Evaluation on real data

3. **"Real-Time OSINT Monitoring: Efficient Change Detection in Public Data Sources"**
   - Monitoring architecture
   - Change detection algorithms
   - Scalability evaluation

---

## 🧪 Complete Testing Suite

### Testing Philosophy

**Comprehensive Coverage**: Every function, API, data structure, backend service, and UI component must be tested.

**Testing Pyramid**:
```
        /\
       /  \     E2E Tests (10%)
      /____\    
     /      \   Integration Tests (30%)
    /________\  
   /          \ Unit Tests (60%)
  /____________\
```

**Principles**:
- **Test-Driven Development**: Write tests before implementation
- **Fast Feedback**: Tests should run quickly (< 5 minutes for full suite)
- **Isolated Tests**: Each test is independent
- **Deterministic**: Tests produce same results every time
- **Comprehensive**: Cover happy paths, edge cases, error cases

---

### 1. Unit Tests

#### 1.1 Core Data Structures

**Entity Tests**:
```rust
#[cfg(test)]
mod entity_tests {
    use super::*;
    
    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(
            EntityType::Domain { domain: "example.com".to_string() },
            vec![SourceInfo::new("WHOIS", now())],
        );
        assert_eq!(entity.entity_type.domain(), "example.com");
        assert!(!entity.id.is_empty());
    }
    
    #[test]
    fn test_entity_confidence_calculation() {
        let entity = create_test_entity();
        let confidence = entity.calculate_confidence();
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }
    
    #[test]
    fn test_entity_serialization() {
        let entity = create_test_entity();
        let json = serde_json::to_string(&entity).unwrap();
        let deserialized: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(entity.id, deserialized.id);
    }
    
    #[test]
    fn test_entity_type_variants() {
        // Test all entity type variants
        let domain = EntityType::Domain { domain: "test.com".to_string() };
        let email = EntityType::Email { email: "test@test.com".to_string(), domain: "test.com".to_string() };
        let person = EntityType::Person { name: "John Doe".to_string(), emails: vec![] };
        // ... test all variants
    }
}
```

**Relationship Tests**:
```rust
#[cfg(test)]
mod relationship_tests {
    use super::*;
    
    #[test]
    fn test_relationship_creation() {
        let from = create_test_entity();
        let to = create_test_entity();
        let relationship = Relationship::new(
            from.id,
            to.id,
            RelationshipType::UsesEmailDomain,
            0.9,
        );
        assert_eq!(relationship.from, from.id);
        assert_eq!(relationship.confidence, 0.9);
    }
    
    #[test]
    fn test_relationship_bidirectional() {
        let rel = create_test_relationship();
        let reverse = rel.reverse();
        assert_eq!(reverse.from, rel.to);
        assert_eq!(reverse.to, rel.from);
    }
    
    #[test]
    fn test_relationship_confidence_threshold() {
        let rel = Relationship::new(EntityId::new(), EntityId::new(), RelationshipType::NameMatch, 0.5);
        assert!(!rel.is_high_confidence()); // Below 0.7 threshold
        
        let rel_high = Relationship::new(EntityId::new(), EntityId::new(), RelationshipType::NameMatch, 0.9);
        assert!(rel_high.is_high_confidence());
    }
}
```

#### 1.2 Algorithm Tests

**Relationship Detection Tests**:
```rust
#[cfg(test)]
mod relationship_detection_tests {
    use super::*;
    
    #[test]
    fn test_email_domain_matching() {
        let email_entity = Entity {
            entity_type: EntityType::Email {
                email: "test@example.com".to_string(),
                domain: "example.com".to_string(),
            },
            // ...
        };
        
        let domain_entity = Entity {
            entity_type: EntityType::Domain {
                domain: "example.com".to_string(),
            },
            // ...
        };
        
        let graph = create_test_graph(vec![email_entity.clone(), domain_entity.clone()]);
        let relationships = detect_relationships(&email_entity, &graph);
        
        assert!(relationships.iter().any(|r| 
            r.relationship_type == RelationshipType::UsesEmailDomain &&
            r.to == domain_entity.id
        ));
    }
    
    #[test]
    fn test_username_matching() {
        let twitter = Entity {
            entity_type: EntityType::Twitter {
                username: "johndoe".to_string(),
                user_id: Some("12345".to_string()),
            },
            // ...
        };
        
        let github = Entity {
            entity_type: EntityType::GitHub {
                username: "johndoe".to_string(),
            },
            // ...
        };
        
        let graph = create_test_graph(vec![twitter.clone(), github.clone()]);
        let relationships = detect_relationships(&twitter, &graph);
        
        assert!(relationships.iter().any(|r| 
            r.relationship_type == RelationshipType::UsernameMatch &&
            r.to == github.id &&
            r.confidence > 0.8
        ));
    }
    
    #[test]
    fn test_name_fuzzy_matching() {
        let entity1 = Entity {
            entity_type: EntityType::Person {
                name: "John Smith".to_string(),
                emails: vec![],
            },
            // ...
        };
        
        let entity2 = Entity {
            entity_type: EntityType::Person {
                name: "J. Smith".to_string(),
                emails: vec![],
            },
            // ...
        };
        
        let graph = create_test_graph(vec![entity1.clone(), entity2.clone()]);
        let relationships = detect_relationships(&entity1, &graph);
        
        assert!(relationships.iter().any(|r| 
            r.relationship_type == RelationshipType::NameMatch &&
            r.to == entity2.id &&
            r.confidence > 0.7
        ));
    }
    
    #[test]
    fn test_no_false_positives() {
        // Test that unrelated entities don't get matched
        let entity1 = create_test_entity();
        let entity2 = create_test_entity_unrelated();
        let graph = create_test_graph(vec![entity1.clone(), entity2.clone()]);
        let relationships = detect_relationships(&entity1, &graph);
        
        assert!(relationships.iter().all(|r| r.to != entity2.id || r.confidence < 0.5));
    }
}
```

**Confidence Scoring Tests**:
```rust
#[cfg(test)]
mod confidence_tests {
    use super::*;
    
    #[test]
    fn test_source_reliability_scoring() {
        let high_quality = SourceInfo::new("WHOIS", now());
        let low_quality = SourceInfo::new("Unknown", now());
        
        assert!(get_source_reliability_score(&high_quality) > 0.8);
        assert!(get_source_reliability_score(&low_quality) < 0.5);
    }
    
    #[test]
    fn test_freshness_scoring() {
        let recent = Entity {
            last_seen: now() - Duration::from_hours(1),
            // ...
        };
        
        let old = Entity {
            last_seen: now() - Duration::from_days(100),
            // ...
        };
        
        assert!(calculate_freshness(&recent.last_seen) > 0.9);
        assert!(calculate_freshness(&old.last_seen) < 0.5);
    }
    
    #[test]
    fn test_cross_source_verification() {
        let entity = Entity {
            sources: vec![
                SourceInfo::new("WHOIS", now()),
                SourceInfo::new("DNS", now()),
                SourceInfo::new("Twitter", now()),
            ],
            // ...
        };
        
        let graph = create_test_graph(vec![entity.clone()]);
        let verification = count_verifying_sources(&entity, &graph);
        assert_eq!(verification, 3);
    }
    
    #[test]
    fn test_confidence_weighted_average() {
        let entity = create_test_entity();
        let graph = create_test_graph(vec![entity.clone()]);
        let confidence = calculate_confidence(&entity, &graph);
        
        // Confidence should be between 0 and 1
        assert!(confidence >= 0.0 && confidence <= 1.0);
        
        // Should be calculated correctly
        let expected = (
            0.3 * get_source_reliability_score(&entity.sources[0]) +
            0.2 * calculate_freshness(&entity.last_seen) +
            0.3 * (count_verifying_sources(&entity, &graph) as f64 / 3.0) +
            0.2 * match_known_patterns(&entity, &graph)
        );
        assert!((confidence - expected).abs() < 0.01);
    }
}
```

**Pattern Matching Tests**:
```rust
#[cfg(test)]
mod pattern_tests {
    use super::*;
    
    #[test]
    fn test_new_domain_pattern() {
        let entity = Entity {
            entity_type: EntityType::Domain {
                domain: "newdomain.com".to_string(),
            },
            created_at: now() - Duration::from_days(3),
            // ...
        };
        
        let pattern = get_pattern_library().iter()
            .find(|p| p.id == "new_domain")
            .unwrap();
        
        let score = pattern.matches(&entity);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.7);
    }
    
    #[test]
    fn test_suspicious_email_pattern() {
        let entity = Entity {
            entity_type: EntityType::Email {
                email: "test@tempmail.com".to_string(),
                domain: "tempmail.com".to_string(),
            },
            // ...
        };
        
        let pattern = get_pattern_library().iter()
            .find(|p| p.id == "suspicious_email")
            .unwrap();
        
        let score = pattern.matches(&entity);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.6);
    }
    
    #[test]
    fn test_pattern_no_match() {
        let entity = create_test_entity_unrelated();
        let patterns = get_pattern_library();
        
        for pattern in patterns {
            let score = pattern.matches(&entity);
            // Should not match unrelated entities
            assert!(score.is_none() || score.unwrap() < 0.5);
        }
    }
}
```

#### 1.3 Query Language Tests

```rust
#[cfg(test)]
mod query_tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let query_str = r#"
            query {
                find: Domain
                where: { domain = "example.com" }
            }
        "#;
        
        let query = parse_query(query_str).unwrap();
        assert_eq!(query.entity_type, Some(EntityType::Domain));
    }
    
    #[test]
    fn test_query_execution() {
        let graph = create_test_graph_with_data();
        let query = Query {
            entity_type: Some(EntityType::Domain),
            filters: vec![Filter::DomainEquals("example.com".to_string())],
            // ...
        };
        
        let results = execute_query(&query, &graph).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().all(|e| matches!(e.entity_type, EntityType::Domain { .. })));
    }
    
    #[test]
    fn test_query_with_relationships() {
        let query = Query {
            expand: Some(ExpandOptions {
                relationships: RelationshipFilter::All,
                depth: 2,
            }),
            // ...
        };
        
        let results = execute_query(&query, &graph).unwrap();
        // Verify relationships are expanded
        for entity in results {
            assert!(!entity.relationships.is_empty());
        }
    }
    
    #[test]
    fn test_query_confidence_filter() {
        let query = Query {
            filters: vec![Filter::ConfidenceGreaterThan(0.7)],
            // ...
        };
        
        let results = execute_query(&query, &graph).unwrap();
        assert!(results.iter().all(|e| e.confidence > 0.7));
    }
}
```

---

### 2. Integration Tests

#### 2.1 Database Integration Tests

```rust
#[cfg(test)]
mod database_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();
        assert!(db_path.exists());
    }
    
    #[test]
    fn test_entity_storage() {
        let db = create_test_database();
        let entity = create_test_entity();
        
        db.store_entity(&entity).unwrap();
        let retrieved = db.get_entity(&entity.id).unwrap();
        assert_eq!(entity.id, retrieved.id);
    }
    
    #[test]
    fn test_relationship_storage() {
        let db = create_test_database();
        let relationship = create_test_relationship();
        
        db.store_relationship(&relationship).unwrap();
        let retrieved = db.get_relationship(&relationship.id).unwrap();
        assert_eq!(relationship.id, retrieved.id);
    }
    
    #[test]
    fn test_graph_traversal() {
        let db = create_test_database();
        let entities = create_test_graph_data();
        
        for entity in &entities {
            db.store_entity(entity).unwrap();
        }
        
        let relationships = db.get_relationships_for_entity(&entities[0].id, 2).unwrap();
        assert!(!relationships.is_empty());
    }
    
    #[test]
    fn test_full_text_search() {
        let db = create_test_database();
        let entity = Entity {
            entity_type: EntityType::Person {
                name: "John Doe".to_string(),
                emails: vec!["john@example.com".to_string()],
            },
            // ...
        };
        
        db.store_entity(&entity).unwrap();
        
        let results = db.search("John").unwrap();
        assert!(results.iter().any(|e| e.id == entity.id));
    }
    
    #[test]
    fn test_database_transactions() {
        let db = create_test_database();
        
        db.begin_transaction().unwrap();
        let entity1 = create_test_entity();
        let entity2 = create_test_entity();
        db.store_entity(&entity1).unwrap();
        db.store_entity(&entity2).unwrap();
        db.rollback().unwrap();
        
        // Entities should not exist after rollback
        assert!(db.get_entity(&entity1.id).is_err());
    }
    
    #[test]
    fn test_database_migration() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // Create old schema
        let db = Database::new_with_schema(&db_path, OLD_SCHEMA).unwrap();
        
        // Migrate to new schema
        db.migrate(NEW_SCHEMA).unwrap();
        
        // Verify migration
        let schema_version = db.get_schema_version().unwrap();
        assert_eq!(schema_version, NEW_SCHEMA_VERSION);
    }
}
```

#### 2.2 Source Integration Tests

```rust
#[cfg(test)]
mod source_integration_tests {
    use super::*;
    use mockito::Server;
    
    #[tokio::test]
    async fn test_rss_source_collection() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/feed.xml")
            .with_status(200)
            .with_body(r#"<?xml version="1.0"?>
                <rss><channel>
                    <item><title>Test Article</title><link>http://example.com/article</link></item>
                </channel></rss>"#)
            .create();
        
        let source = RSSSource::new(&server.url());
        let query = Query::new(EntityType::Article);
        let entities = source.collect(&query).await.unwrap();
        
        assert!(!entities.is_empty());
        mock.assert();
    }
    
    #[tokio::test]
    async fn test_whois_source_collection() {
        let source = WHOISSource::new();
        let query = Query::new(EntityType::Domain)
            .with_filter(Filter::DomainEquals("example.com".to_string()));
        
        let entities = source.collect(&query).await.unwrap();
        assert!(!entities.is_empty());
        assert!(entities.iter().any(|e| matches!(e.entity_type, EntityType::Domain { domain } if domain == "example.com")));
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let source = TwitterSource::new_with_credentials(test_credentials());
        let rate_limiter = source.rate_limit();
        
        // Make requests up to limit
        for _ in 0..rate_limiter.max_requests {
            let _ = source.collect(&Query::new(EntityType::Twitter)).await;
        }
        
        // Next request should be rate limited
        let start = Instant::now();
        let _ = source.collect(&Query::new(EntityType::Twitter)).await;
        let elapsed = start.elapsed();
        
        assert!(elapsed >= rate_limiter.window);
    }
    
    #[tokio::test]
    async fn test_source_error_handling() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/feed.xml")
            .with_status(500)
            .create();
        
        let source = RSSSource::new(&server.url());
        let result = source.collect(&Query::new(EntityType::Article)).await;
        
        assert!(result.is_err());
        mock.assert();
    }
    
    #[tokio::test]
    async fn test_source_retry_logic() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/feed.xml")
            .with_status(500)
            .with_status(500)
            .with_status(200)
            .with_body(r#"<?xml version="1.0"?><rss><channel></channel></rss>"#)
            .create();
        
        let source = RSSSource::new(&server.url())
            .with_retry_policy(RetryPolicy::exponential_backoff(3));
        
        let result = source.collect(&Query::new(EntityType::Article)).await;
        assert!(result.is_ok());
        assert_eq!(mock.hits(), 3);
    }
    
    #[tokio::test]
    async fn test_source_authentication() {
        let source = TwitterSource::new();
        assert!(!source.is_authenticated());
        
        source.authenticate(test_credentials()).await.unwrap();
        assert!(source.is_authenticated());
    }
}
```

#### 2.3 Monitoring Integration Tests

```rust
#[cfg(test)]
mod monitoring_tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_monitor_creation() {
        let source = create_test_source();
        let query = Query::new(EntityType::Domain);
        let monitor = source.monitor(&query).await.unwrap();
        
        assert_eq!(monitor.source, source.name());
        assert_eq!(monitor.query, query);
    }
    
    #[tokio::test]
    async fn test_monitor_polling() {
        let monitor = create_test_monitor();
        let mut monitor_manager = MonitorManager::new();
        
        monitor_manager.add_monitor(monitor).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        
        let alerts = monitor_manager.check_all().await.unwrap();
        // Verify alerts are generated when changes detected
    }
    
    #[tokio::test]
    async fn test_monitor_alert_conditions() {
        let monitor = Monitor {
            alert_conditions: vec![
                AlertCondition {
                    condition: Condition::NewEntity,
                    action: AlertAction::Notify,
                },
            ],
            // ...
        };
        
        // Trigger condition
        let alert = monitor.check().await.unwrap();
        assert!(alert.is_some());
    }
    
    #[tokio::test]
    async fn test_monitor_state_persistence() {
        let monitor = create_test_monitor();
        let state = monitor.get_state();
        
        // Simulate restart
        let restored = Monitor::from_state(state).unwrap();
        assert_eq!(monitor.id, restored.id);
        assert_eq!(monitor.last_state, restored.last_state);
    }
}
```

---

### 3. API Tests (Tauri Commands)

```rust
#[cfg(test)]
mod api_tests {
    use super::*;
    use tauri::test;
    
    #[tokio::test]
    async fn test_query_entities_command() {
        let app = create_test_app().await;
        let state = app.state::<AppState>();
        
        let result = commands::query_entities(
            state,
            QueryRequest {
                entity_type: Some("Domain".to_string()),
                filters: vec![],
            },
        ).await;
        
        assert!(result.is_ok());
        let entities = result.unwrap();
        assert!(!entities.is_empty());
    }
    
    #[tokio::test]
    async fn test_add_source_command() {
        let app = create_test_app().await;
        let state = app.state::<AppState>();
        
        let result = commands::add_source(
            state,
            AddSourceRequest {
                source_type: "RSS".to_string(),
                config: SourceConfig {
                    url: "https://example.com/feed.xml".to_string(),
                },
            },
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_create_monitor_command() {
        let app = create_test_app().await;
        let state = app.state::<AppState>();
        
        let result = commands::create_monitor(
            state,
            CreateMonitorRequest {
                source_id: "rss-1".to_string(),
                query: QueryRequest { /* ... */ },
                poll_interval: 300,
            },
        ).await;
        
        assert!(result.is_ok());
        let monitor = result.unwrap();
        assert!(!monitor.id.is_empty());
    }
    
    #[tokio::test]
    async fn test_detect_relationships_command() {
        let app = create_test_app().await;
        let state = app.state::<AppState>();
        
        let entity_id = create_test_entity_in_db(state).await;
        
        let result = commands::detect_relationships(
            state,
            entity_id,
        ).await;
        
        assert!(result.is_ok());
        let relationships = result.unwrap();
        // Verify relationships detected
    }
    
    #[tokio::test]
    async fn test_export_investigation_command() {
        let app = create_test_app().await;
        let state = app.state::<AppState>();
        
        let investigation_id = create_test_investigation(state).await;
        
        let result = commands::export_investigation(
            state,
            investigation_id,
            ExportFormat::JSON,
        ).await;
        
        assert!(result.is_ok());
        let export_data = result.unwrap();
        assert!(!export_data.is_empty());
    }
    
    #[tokio::test]
    async fn test_api_error_handling() {
        let app = create_test_app().await;
        let state = app.state::<AppState>();
        
        // Test invalid entity ID
        let result = commands::get_entity(
            state,
            EntityId::new(), // Non-existent ID
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::EntityNotFound));
    }
}
```

---

### 4. UI Tests

#### 4.1 Component Tests (Leptos)

```rust
#[cfg(test)]
mod ui_component_tests {
    use super::*;
    use leptos::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_graph_view_component() {
        let entities = create_test_entities();
        let relationships = create_test_relationships();
        
        mount_to_body(|cx| {
            view! { cx,
                <GraphView entities=entities relationships=relationships />
            }
        });
        
        // Verify graph renders
        let graph_element = document().query_selector(".graph-view").unwrap();
        assert!(graph_element.is_some());
    }
    
    #[wasm_bindgen_test]
    fn test_query_builder_component() {
        mount_to_body(|cx| {
            view! { cx,
                <QueryBuilder />
            }
        });
        
        // Test query building
        let query_input = document().query_selector("input[type='text']").unwrap();
        // Simulate user input
        // Verify query is built correctly
    }
    
    #[wasm_bindgen_test]
    fn test_entity_details_component() {
        let entity = create_test_entity();
        
        mount_to_body(|cx| {
            view! { cx,
                <EntityDetails entity=entity />
            }
        });
        
        // Verify entity details render
        let name_element = document().query_selector(".entity-name").unwrap();
        assert!(name_element.is_some());
    }
    
    #[wasm_bindgen_test]
    fn test_source_manager_component() {
        mount_to_body(|cx| {
            view! { cx,
                <SourceManager />
            }
        });
        
        // Test adding source
        // Test removing source
        // Test configuring source
    }
}
```

#### 4.2 UI Integration Tests (Playwright)

```rust
// tests/e2e/ui_tests.rs
use playwright::api::*;

#[tokio::test]
async fn test_graph_view_interaction() {
    let browser = Browser::launch(BrowserType::Chromium).await.unwrap();
    let context = browser.new_context().await.unwrap();
    let page = context.new_page().await.unwrap();
    
    page.goto("http://localhost:1420").await.unwrap();
    
    // Wait for graph to load
    page.wait_for_selector(".graph-view").await.unwrap();
    
    // Test zoom
    page.click(".zoom-in").await.unwrap();
    
    // Test pan
    page.mouse_move(100, 100).await.unwrap();
    page.mouse_down().await.unwrap();
    page.mouse_move(200, 200).await.unwrap();
    page.mouse_up().await.unwrap();
    
    // Test entity click
    page.click(".entity-node").await.unwrap();
    page.wait_for_selector(".entity-details").await.unwrap();
}

#[tokio::test]
async fn test_query_execution() {
    let page = create_test_page().await;
    
    // Enter query
    page.fill("input[data-testid='query-input']", "find: Domain where domain = 'example.com'").await.unwrap();
    
    // Execute query
    page.click("button[data-testid='execute-query']").await.unwrap();
    
    // Wait for results
    page.wait_for_selector(".query-results").await.unwrap();
    
    // Verify results
    let results = page.query_selector_all(".result-item").await.unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_source_management() {
    let page = create_test_page().await;
    
    // Navigate to source manager
    page.click("a[href='/sources']").await.unwrap();
    
    // Add new source
    page.click("button[data-testid='add-source']").await.unwrap();
    page.fill("input[name='url']", "https://example.com/feed.xml").await.unwrap();
    page.select_option("select[name='source-type']", "RSS").await.unwrap();
    page.click("button[type='submit']").await.unwrap();
    
    // Verify source added
    page.wait_for_selector(".source-item").await.unwrap();
}

#[tokio::test]
async fn test_monitor_creation() {
    let page = create_test_page().await;
    
    // Navigate to monitoring
    page.click("a[href='/monitoring']").await.unwrap();
    
    // Create monitor
    page.click("button[data-testid='create-monitor']").await.unwrap();
    page.fill("input[name='query']", "find: Domain").await.unwrap();
    page.fill("input[name='interval']", "300").await.unwrap();
    page.click("button[type='submit']").await.unwrap();
    
    // Verify monitor created
    page.wait_for_selector(".monitor-item").await.unwrap();
}
```

---

### 5. End-to-End Tests

```rust
// tests/e2e/full_workflow_tests.rs

#[tokio::test]
async fn test_complete_investigation_workflow() {
    // 1. Start application
    let app = start_test_app().await;
    
    // 2. Add RSS source
    add_source(&app, SourceConfig::rss("https://example.com/feed.xml")).await;
    
    // 3. Create monitor
    let monitor_id = create_monitor(&app, MonitorConfig {
        source: "rss-1",
        query: Query::new(EntityType::Article),
        interval: 300,
    }).await;
    
    // 4. Wait for data collection
    sleep(Duration::from_secs(2)).await;
    
    // 5. Query entities
    let entities = query_entities(&app, Query::new(EntityType::Article)).await;
    assert!(!entities.is_empty());
    
    // 6. Detect relationships
    for entity in &entities {
        detect_relationships(&app, entity.id).await;
    }
    
    // 7. Visualize graph
    let graph = get_graph(&app, GraphOptions::default()).await;
    assert!(!graph.entities.is_empty());
    assert!(!graph.relationships.is_empty());
    
    // 8. Export investigation
    let export = export_investigation(&app, InvestigationId::new(), ExportFormat::JSON).await;
    assert!(!export.is_empty());
}

#[tokio::test]
async fn test_cross_source_relationship_detection() {
    let app = start_test_app().await;
    
    // Add multiple sources
    add_source(&app, SourceConfig::rss("https://example.com/feed.xml")).await;
    add_source(&app, SourceConfig::whois()).await;
    add_source(&app, SourceConfig::twitter(test_credentials())).await;
    
    // Collect from all sources
    collect_from_source(&app, "rss-1").await;
    collect_from_source(&app, "whois-1").await;
    collect_from_source(&app, "twitter-1").await;
    
    // Detect cross-source relationships
    detect_all_relationships(&app).await;
    
    // Verify relationships found
    let relationships = get_all_relationships(&app).await;
    assert!(!relationships.is_empty());
    
    // Verify cross-source relationships exist
    let cross_source = relationships.iter()
        .filter(|r| is_cross_source_relationship(r))
        .count();
    assert!(cross_source > 0);
}
```

---

### 6. Performance Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_query_performance() {
        let graph = create_large_test_graph(10_000); // 10k entities
        
        let start = Instant::now();
        let results = execute_query(&Query::new(EntityType::Domain), &graph).unwrap();
        let elapsed = start.elapsed();
        
        assert!(elapsed < Duration::from_millis(100)); // Should be fast
        assert!(!results.is_empty());
    }
    
    #[test]
    fn test_relationship_detection_performance() {
        let graph = create_large_test_graph(5_000);
        let entity = graph.get_random_entity().unwrap();
        
        let start = Instant::now();
        let relationships = detect_relationships(&entity, &graph);
        let elapsed = start.elapsed();
        
        assert!(elapsed < Duration::from_millis(500)); // Should complete quickly
        assert!(!relationships.is_empty());
    }
    
    #[test]
    fn test_graph_traversal_performance() {
        let graph = create_large_test_graph(20_000);
        let start_entity = graph.get_random_entity().unwrap();
        
        let start = Instant::now();
        let reachable = graph.traverse_from(&start_entity.id, 3).unwrap();
        let elapsed = start.elapsed();
        
        assert!(elapsed < Duration::from_millis(200));
        assert!(!reachable.is_empty());
    }
    
    #[test]
    fn test_database_insert_performance() {
        let db = create_test_database();
        let entities = create_test_entities(1_000);
        
        let start = Instant::now();
        for entity in entities {
            db.store_entity(&entity).unwrap();
        }
        let elapsed = start.elapsed();
        
        assert!(elapsed < Duration::from_secs(5)); // 1000 inserts in 5 seconds
    }
    
    #[test]
    fn test_memory_usage() {
        let graph = create_large_test_graph(50_000);
        let memory_before = get_memory_usage();
        
        // Perform operations
        for _ in 0..100 {
            let entity = graph.get_random_entity().unwrap();
            let _ = detect_relationships(&entity, &graph);
        }
        
        let memory_after = get_memory_usage();
        let memory_increase = memory_after - memory_before;
        
        // Should not use excessive memory
        assert!(memory_increase < 100 * 1024 * 1024); // < 100MB
    }
}
```

---

### 7. Security Tests

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_sql_injection_prevention() {
        let db = create_test_database();
        let malicious_query = "'; DROP TABLE entities; --";
        
        // Should be sanitized
        let result = db.query(&format!("SELECT * FROM entities WHERE name = '{}'", malicious_query));
        assert!(result.is_err() || !result.unwrap().is_empty()); // Should not execute DROP
    }
    
    #[test]
    fn test_authentication_required() {
        let source = TwitterSource::new();
        
        // Should fail without authentication
        let result = source.collect(&Query::new(EntityType::Twitter)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::AuthenticationRequired));
    }
    
    #[test]
    fn test_encryption() {
        let data = b" sensitive data ";
        let key = generate_encryption_key();
        
        let encrypted = encrypt(data, &key).unwrap();
        assert_ne!(encrypted, data); // Should be different
        
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, data); // Should decrypt correctly
    }
    
    #[test]
    fn test_rate_limit_enforcement() {
        let source = create_test_source();
        let rate_limiter = source.rate_limit();
        
        // Exceed rate limit
        for _ in 0..rate_limiter.max_requests + 10 {
            let result = source.collect(&Query::new(EntityType::Domain)).await;
            if result.is_err() {
                assert!(matches!(result.unwrap_err(), Error::RateLimitExceeded));
                break;
            }
        }
    }
    
    #[test]
    fn test_input_validation() {
        // Test malicious inputs
        let malicious_inputs = vec![
            "../../etc/passwd",
            "<script>alert('xss')</script>",
            "'; DROP TABLE entities; --",
            "\0",
        ];
        
        for input in malicious_inputs {
            let result = validate_input(input);
            assert!(result.is_err() || !result.unwrap()); // Should reject or sanitize
        }
    }
}
```

---

### 8. Data Validation Tests

```rust
#[cfg(test)]
mod data_validation_tests {
    use super::*;
    
    #[test]
    fn test_entity_validation() {
        // Valid entity
        let valid = Entity {
            entity_type: EntityType::Domain {
                domain: "example.com".to_string(),
            },
            // ...
        };
        assert!(valid.validate().is_ok());
        
        // Invalid entity (empty domain)
        let invalid = Entity {
            entity_type: EntityType::Domain {
                domain: "".to_string(),
            },
            // ...
        };
        assert!(invalid.validate().is_err());
    }
    
    #[test]
    fn test_email_validation() {
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.com",
        ];
        
        for email in valid_emails {
            assert!(validate_email(email));
        }
        
        let invalid_emails = vec![
            "not-an-email",
            "@example.com",
            "user@",
            "user@.com",
        ];
        
        for email in invalid_emails {
            assert!(!validate_email(email));
        }
    }
    
    #[test]
    fn test_domain_validation() {
        let valid_domains = vec![
            "example.com",
            "sub.example.com",
            "example.co.uk",
        ];
        
        for domain in valid_domains {
            assert!(validate_domain(domain));
        }
        
        let invalid_domains = vec![
            "",
            "not a domain",
            ".com",
            "example.",
        ];
        
        for domain in invalid_domains {
            assert!(!validate_domain(domain));
        }
    }
    
    #[test]
    fn test_confidence_range_validation() {
        let entity = Entity {
            confidence: 1.5, // Invalid: > 1.0
            // ...
        };
        
        assert!(entity.validate().is_err());
        
        let entity = Entity {
            confidence: -0.1, // Invalid: < 0.0
            // ...
        };
        
        assert!(entity.validate().is_err());
    }
}
```

---

### 9. Test Infrastructure

#### 9.1 Test Utilities

```rust
// tests/utils/mod.rs

pub mod test_helpers {
    use super::*;
    
    pub fn create_test_entity() -> Entity {
        Entity {
            id: EntityId::new(),
            entity_type: EntityType::Domain {
                domain: "test.example.com".to_string(),
            },
            sources: vec![SourceInfo::new("TEST", now())],
            relationships: vec![],
            confidence: 0.8,
            created_at: now(),
            updated_at: now(),
            last_seen: now(),
        }
    }
    
    pub fn create_test_graph(size: usize) -> Graph {
        let mut graph = Graph::new();
        for _ in 0..size {
            let entity = create_test_entity();
            graph.add_entity(entity);
        }
        graph
    }
    
    pub fn create_test_database() -> Database {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Database::new(&db_path).unwrap()
    }
    
    pub async fn create_test_app() -> tauri::App {
        // Create test Tauri app instance
        // ...
    }
}
```

#### 9.2 Mock Sources

```rust
// tests/mocks/mod.rs

pub struct MockRSSSource {
    responses: Vec<String>,
    current: usize,
}

impl MockRSSSource {
    pub fn new() -> Self {
        Self {
            responses: vec![],
            current: 0,
        }
    }
    
    pub fn with_response(mut self, response: String) -> Self {
        self.responses.push(response);
        self
    }
}

impl Source for MockRSSSource {
    fn collect(&self, query: &Query) -> Result<Vec<Entity>> {
        // Return mock entities
        Ok(vec![create_test_entity()])
    }
    // ...
}
```

#### 9.3 Test Fixtures

```rust
// tests/fixtures/mod.rs

pub mod fixtures {
    pub const TEST_RSS_FEED: &str = r#"
        <?xml version="1.0"?>
        <rss>
            <channel>
                <item>
                    <title>Test Article</title>
                    <link>http://example.com/article</link>
                </item>
            </channel>
        </rss>
    "#;
    
    pub const TEST_WHOIS_RESPONSE: &str = r#"
        Domain: example.com
        Registrar: Test Registrar
        Created: 2024-01-01
    "#;
    
    // ... more fixtures
}
```

---

### 10. Continuous Testing

#### 10.1 Test Configuration

```toml
# Cargo.toml
[dev-dependencies]
tokio-test = "0.4"
mockito = "1.0"
tempfile = "3.0"
wasm-bindgen-test = "0.3"
playwright = "0.1"

[[test]]
name = "unit"
path = "tests/unit/mod.rs"

[[test]]
name = "integration"
path = "tests/integration/mod.rs"

[[test]]
name = "api"
path = "tests/api/mod.rs"

[[test]]
name = "e2e"
path = "tests/e2e/mod.rs"
```

#### 10.2 CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test integration
      
      - name: Run API tests
        run: cargo test --test api
      
      - name: Run E2E tests
        run: cargo test --test e2e
      
      - name: Generate coverage
        run: cargo test --coverage
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

### 11. Test Coverage Requirements

**Minimum Coverage Targets**:
- **Unit Tests**: 90%+ coverage
- **Integration Tests**: 80%+ coverage
- **API Tests**: 100% of public APIs
- **UI Tests**: All critical user flows
- **E2E Tests**: All major workflows

**Coverage Tools**:
- `cargo-tarpaulin` for Rust code coverage
- `cargo-llvm-cov` for LLVM-based coverage
- Coverage reports in CI/CD

---

### 12. Test Execution

**Local Development**:
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration

# Run with coverage
cargo test --coverage

# Run UI tests
cargo test --test ui --features ui-tests

# Run E2E tests
cargo test --test e2e --features e2e-tests
```

**CI/CD**:
- All tests run on every commit
- Tests must pass before merge
- Coverage reports generated
- Performance benchmarks tracked

---

## 📊 Test Metrics

### Coverage Metrics
- **Line Coverage**: Percentage of code lines executed
- **Branch Coverage**: Percentage of branches tested
- **Function Coverage**: Percentage of functions tested

### Quality Metrics
- **Test Execution Time**: Should complete in < 5 minutes
- **Test Reliability**: 100% pass rate (no flaky tests)
- **Test Maintainability**: Tests should be easy to update

### Performance Metrics
- **Query Performance**: < 100ms for simple queries
- **Relationship Detection**: < 500ms for 1000 entities
- **Database Operations**: < 5s for 1000 inserts

---

## 🎯 Testing Best Practices

1. **Test Isolation**: Each test is independent
2. **Fast Tests**: Unit tests should be very fast (< 1ms each)
3. **Clear Test Names**: Test names describe what they test
4. **Arrange-Act-Assert**: Clear test structure
5. **Test Data**: Use fixtures, not production data
6. **Mock External Services**: Don't hit real APIs in tests
7. **Test Edge Cases**: Test boundaries, errors, nulls
8. **Maintain Tests**: Update tests when code changes

---

## 🎉 Summary

**Complete Testing Suite** covers:
- ✅ **Unit Tests**: All functions, algorithms, data structures
- ✅ **Integration Tests**: Database, sources, monitoring
- ✅ **API Tests**: All Tauri commands
- ✅ **UI Tests**: All components and interactions
- ✅ **E2E Tests**: Complete workflows
- ✅ **Performance Tests**: Speed and memory
- ✅ **Security Tests**: Security vulnerabilities
- ✅ **Data Validation**: Input validation
- ✅ **Test Infrastructure**: Utilities, mocks, fixtures
- ✅ **Continuous Testing**: CI/CD integration

**Result**: Comprehensive test coverage ensuring reliability, correctness, and maintainability of the entire OSIRIS platform.

---

---

## 📚 Complete Documentation System

### Documentation Philosophy

**Comprehensive Coverage**: Every function, API, component, algorithm, and feature must be documented.

**Documentation Types**:
- **API Documentation**: Every function, struct, enum, trait
- **User Documentation**: How to use every feature
- **Developer Documentation**: How to extend and contribute
- **Architecture Documentation**: System design and decisions
- **Tutorial Documentation**: Step-by-step guides
- **Reference Documentation**: Complete reference for everything

**Documentation Stack**:
- **Docusaurus**: Main documentation site
- **Rust Doc**: Auto-generated API docs from code
- **Type System**: Type-safe documentation
- **Examples**: Code examples for everything

---

### 1. Docusaurus Setup

#### 1.1 Project Structure

```
docs/
├── docusaurus.config.js
├── package.json
├── sidebars.js
├── src/
│   ├── css/
│   │   └── custom.css
│   └── pages/
│       ├── index.tsx
│       └── ...
├── docs/
│   ├── intro.md
│   ├── getting-started/
│   │   ├── installation.md
│   │   ├── quick-start.md
│   │   └── configuration.md
│   ├── user-guide/
│   │   ├── overview.md
│   │   ├── entities.md
│   │   ├── relationships.md
│   │   ├── queries.md
│   │   ├── sources/
│   │   │   ├── overview.md
│   │   │   ├── rss.md
│   │   │   ├── whois.md
│   │   │   ├── twitter.md
│   │   │   └── ...
│   │   ├── monitoring.md
│   │   ├── investigations.md
│   │   └── visualization.md
│   ├── api-reference/
│   │   ├── overview.md
│   │   ├── entities/
│   │   │   ├── entity.md
│   │   │   ├── entity-type.md
│   │   │   └── ...
│   │   ├── relationships/
│   │   │   ├── relationship.md
│   │   │   ├── relationship-type.md
│   │   │   └── detection.md
│   │   ├── queries/
│   │   │   ├── query.md
│   │   │   ├── query-language.md
│   │   │   └── execution.md
│   │   ├── sources/
│   │   │   ├── source-trait.md
│   │   │   ├── rss-source.md
│   │   │   ├── whois-source.md
│   │   │   └── ...
│   │   ├── monitoring/
│   │   │   ├── monitor.md
│   │   │   ├── alerts.md
│   │   │   └── ...
│   │   ├── database/
│   │   │   ├── database.md
│   │   │   ├── storage.md
│   │   │   └── queries.md
│   │   └── commands/
│   │       ├── query-entities.md
│   │       ├── add-source.md
│   │       ├── create-monitor.md
│   │       └── ...
│   ├── developer-guide/
│   │   ├── architecture/
│   │   │   ├── overview.md
│   │   │   ├── data-model.md
│   │   │   ├── graph-structure.md
│   │   │   ├── source-abstraction.md
│   │   │   └── ...
│   │   ├── extending/
│   │   │   ├── adding-sources.md
│   │   │   ├── custom-relationships.md
│   │   │   ├── plugins.md
│   │   │   └── ...
│   │   ├── contributing/
│   │   │   ├── setup.md
│   │   │   ├── code-style.md
│   │   │   ├── testing.md
│   │   │   └── pull-requests.md
│   │   └── internals/
│   │       ├── algorithms.md
│   │       ├── performance.md
│   │       └── ...
│   ├── tutorials/
│   │   ├── first-investigation.md
│   │   ├── cross-source-analysis.md
│   │   ├── monitoring-setup.md
│   │   └── ...
│   ├── examples/
│   │   ├── basic-query.md
│   │   ├── relationship-detection.md
│   │   ├── custom-source.md
│   │   └── ...
│   └── reference/
│       ├── query-language.md
│       ├── entity-types.md
│       ├── relationship-types.md
│       └── ...
└── static/
    └── img/
```

#### 1.2 Docusaurus Configuration

```javascript
// docs/docusaurus.config.js
const {themes} = require('@docusaurus/preset-classic');

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'OSIRIS Documentation',
  tagline: 'Privacy-First Unified OSINT Platform',
  url: 'https://osiris.osint',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  favicon: 'img/favicon.ico',
  organizationName: 'osiris',
  projectName: 'osiris-docs',
  
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/osiris/osiris/tree/main/docs/',
          showLastUpdateAuthor: true,
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
  
  themeConfig: {
    navbar: {
      title: 'OSIRIS',
      logo: {
        alt: 'OSIRIS Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'doc',
          docId: 'intro',
          position: 'left',
          label: 'Docs',
        },
        {
          type: 'docSidebar',
          sidebarId: 'api',
          position: 'left',
          label: 'API Reference',
        },
        {
          type: 'docSidebar',
          sidebarId: 'tutorials',
          position: 'left',
          label: 'Tutorials',
        },
        {
          href: 'https://github.com/osiris/osiris',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Getting Started',
              to: '/docs/getting-started/installation',
            },
            {
              label: 'User Guide',
              to: '/docs/user-guide/overview',
            },
            {
              label: 'API Reference',
              to: '/docs/api-reference/overview',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/osiris/osiris',
            },
            {
              label: 'Discussions',
              href: 'https://github.com/osiris/osiris/discussions',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} OSIRIS Project.`,
    },
    algolia: {
      appId: 'YOUR_APP_ID',
      apiKey: 'YOUR_SEARCH_API_KEY',
      indexName: 'osiris',
      contextualSearch: true,
    },
  },
};

module.exports = config;
```

#### 1.3 Sidebar Configuration

```javascript
// docs/sidebars.js
module.exports = {
  docs: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        'getting-started/installation',
        'getting-started/quick-start',
        'getting-started/configuration',
      ],
    },
    {
      type: 'category',
      label: 'User Guide',
      items: [
        'user-guide/overview',
        'user-guide/entities',
        'user-guide/relationships',
        'user-guide/queries',
        {
          type: 'category',
          label: 'Sources',
          items: [
            'user-guide/sources/overview',
            'user-guide/sources/rss',
            'user-guide/sources/whois',
            'user-guide/sources/twitter',
            'user-guide/sources/linkedin',
            'user-guide/sources/github',
            'user-guide/sources/dns',
          ],
        },
        'user-guide/monitoring',
        'user-guide/investigations',
        'user-guide/visualization',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      collapsed: false,
      items: [
        'api-reference/overview',
        {
          type: 'category',
          label: 'Entities',
          items: [
            'api-reference/entities/entity',
            'api-reference/entities/entity-type',
            'api-reference/entities/entity-id',
            'api-reference/entities/confidence',
          ],
        },
        {
          type: 'category',
          label: 'Relationships',
          items: [
            'api-reference/relationships/relationship',
            'api-reference/relationships/relationship-type',
            'api-reference/relationships/detection',
            'api-reference/relationships/confidence',
          ],
        },
        {
          type: 'category',
          label: 'Queries',
          items: [
            'api-reference/queries/query',
            'api-reference/queries/query-language',
            'api-reference/queries/execution',
            'api-reference/queries/filters',
          ],
        },
        {
          type: 'category',
          label: 'Sources',
          items: [
            'api-reference/sources/source-trait',
            'api-reference/sources/rss-source',
            'api-reference/sources/whois-source',
            'api-reference/sources/twitter-source',
            'api-reference/sources/linkedin-source',
            'api-reference/sources/github-source',
            'api-reference/sources/dns-source',
          ],
        },
        {
          type: 'category',
          label: 'Monitoring',
          items: [
            'api-reference/monitoring/monitor',
            'api-reference/monitoring/alerts',
            'api-reference/monitoring/conditions',
          ],
        },
        {
          type: 'category',
          label: 'Database',
          items: [
            'api-reference/database/database',
            'api-reference/database/storage',
            'api-reference/database/queries',
            'api-reference/database/transactions',
          ],
        },
        {
          type: 'category',
          label: 'Tauri Commands',
          items: [
            'api-reference/commands/query-entities',
            'api-reference/commands/add-source',
            'api-reference/commands/create-monitor',
            'api-reference/commands/detect-relationships',
            'api-reference/commands/export-investigation',
            // ... all commands
          ],
        },
      ],
    },
    {
      type: 'category',
      label: 'Developer Guide',
      items: [
        {
          type: 'category',
          label: 'Architecture',
          items: [
            'developer-guide/architecture/overview',
            'developer-guide/architecture/data-model',
            'developer-guide/architecture/graph-structure',
            'developer-guide/architecture/source-abstraction',
            'developer-guide/architecture/monitoring',
          ],
        },
        {
          type: 'category',
          label: 'Extending',
          items: [
            'developer-guide/extending/adding-sources',
            'developer-guide/extending/custom-relationships',
            'developer-guide/extending/plugins',
          ],
        },
        {
          type: 'category',
          label: 'Contributing',
          items: [
            'developer-guide/contributing/setup',
            'developer-guide/contributing/code-style',
            'developer-guide/contributing/testing',
            'developer-guide/contributing/pull-requests',
          ],
        },
        {
          type: 'category',
          label: 'Internals',
          items: [
            'developer-guide/internals/algorithms',
            'developer-guide/internals/performance',
            'developer-guide/internals/security',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: 'Tutorials',
      items: [
        'tutorials/first-investigation',
        'tutorials/cross-source-analysis',
        'tutorials/monitoring-setup',
        'tutorials/custom-source',
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      items: [
        'examples/basic-query',
        'examples/relationship-detection',
        'examples/custom-source',
        'examples/monitoring',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/query-language',
        'reference/entity-types',
        'reference/relationship-types',
        'reference/error-codes',
      ],
    },
  ],
};
```

---

### 2. API Documentation (Every Function)

#### 2.1 Rust Doc Comments

Every function, struct, enum, and trait must have comprehensive documentation:

```rust
/// Represents an entity in the OSINT graph.
///
/// An entity is a node in the graph that represents a real-world object
/// such as a person, domain, IP address, email, or social media account.
///
/// # Examples
///
/// ```rust
/// use osiris::Entity;
///
/// let entity = Entity::new(
///     EntityType::Domain {
///         domain: "example.com".to_string(),
///     },
///     vec![SourceInfo::new("WHOIS", now())],
/// );
///
/// assert_eq!(entity.entity_type.domain(), Some("example.com"));
/// ```
///
/// # Fields
///
/// - `id`: Unique identifier for the entity
/// - `entity_type`: The type and data of the entity
/// - `sources`: List of sources that provided this entity
/// - `relationships`: List of relationship IDs connected to this entity
/// - `confidence`: Confidence score (0.0 to 1.0)
/// - `created_at`: Timestamp when entity was first created
/// - `updated_at`: Timestamp when entity was last updated
/// - `last_seen`: Timestamp when entity was last seen in any source
///
/// # See Also
///
/// - [`EntityType`]: Types of entities
/// - [`Relationship`]: Connections between entities
/// - [`SourceInfo`]: Information about data sources
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier for this entity
    pub id: EntityId,
    
    /// The type and data of the entity
    pub entity_type: EntityType,
    
    /// Sources that provided information about this entity
    pub sources: Vec<SourceInfo>,
    
    /// Relationships connected to this entity
    pub relationships: Vec<RelationshipId>,
    
    /// Confidence score (0.0 to 1.0)
    /// 
    /// Higher values indicate more reliable data.
    /// Calculated based on source reliability, data freshness,
    /// cross-source verification, and pattern matching.
    pub confidence: f64,
    
    /// Timestamp when entity was first created
    pub created_at: Timestamp,
    
    /// Timestamp when entity was last updated
    pub updated_at: Timestamp,
    
    /// Timestamp when entity was last seen in any source
    pub last_seen: Timestamp,
}

impl Entity {
    /// Creates a new entity with the given type and sources.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - The type and data of the entity
    /// * `sources` - List of sources that provided this entity
    ///
    /// # Returns
    ///
    /// A new `Entity` with:
    /// - Generated unique ID
    /// - Current timestamp for created_at, updated_at, and last_seen
    /// - Initial confidence score of 0.5
    /// - Empty relationships list
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osiris::{Entity, EntityType, SourceInfo};
    ///
    /// let entity = Entity::new(
    ///     EntityType::Domain {
    ///         domain: "example.com".to_string(),
    ///     },
    ///     vec![SourceInfo::new("WHOIS", now())],
    /// );
    ///
    /// assert!(!entity.id.is_empty());
    /// assert_eq!(entity.confidence, 0.5);
    /// ```
    pub fn new(entity_type: EntityType, sources: Vec<SourceInfo>) -> Self {
        // Implementation
    }
    
    /// Calculates the confidence score for this entity.
    ///
    /// The confidence score is calculated based on:
    /// - Source reliability (30% weight)
    /// - Data freshness (20% weight)
    /// - Cross-source verification (30% weight)
    /// - Pattern matching (20% weight)
    ///
    /// # Arguments
    ///
    /// * `graph` - The graph containing related entities for verification
    ///
    /// # Returns
    ///
    /// A confidence score between 0.0 and 1.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osiris::{Entity, Graph};
    ///
    /// let entity = create_test_entity();
    /// let graph = create_test_graph();
    ///
    /// let confidence = entity.calculate_confidence(&graph);
    /// assert!(confidence >= 0.0 && confidence <= 1.0);
    /// ```
    ///
    /// # See Also
    ///
    /// - [`get_source_reliability_score`]: Source reliability calculation
    /// - [`count_verifying_sources`]: Cross-source verification
    /// - [`match_known_patterns`]: Pattern matching
    pub fn calculate_confidence(&self, graph: &Graph) -> f64 {
        // Implementation
    }
    
    /// Validates the entity data.
    ///
    /// Checks that:
    /// - Entity type is valid
    /// - Confidence is between 0.0 and 1.0
    /// - Sources are not empty
    /// - Timestamps are valid
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(ValidationError)` if invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use osiris::Entity;
    ///
    /// let entity = create_test_entity();
    /// assert!(entity.validate().is_ok());
    ///
    /// let invalid = Entity {
    ///     confidence: 1.5, // Invalid: > 1.0
    ///     // ...
    /// };
    /// assert!(invalid.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Implementation
    }
}
```

#### 2.2 Function Documentation Template

Every function should follow this template:

```rust
/// [Brief one-line description]
///
/// [Detailed description explaining what the function does,
/// why it exists, and how it works.]
///
/// # Arguments
///
/// * `param1` - [Description of parameter 1]
/// * `param2` - [Description of parameter 2]
///
/// # Returns
///
/// [Description of return value]
///
/// # Errors
///
/// [List of errors that can occur]
///
/// # Examples
///
/// ```rust
/// [Code example showing how to use the function]
/// ```
///
/// # Performance
///
/// [Performance characteristics if relevant]
///
/// # See Also
///
/// - [`RelatedFunction`]: Related functions
/// - [`RelatedType`]: Related types
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, Error> {
    // Implementation
}
```

---

### 3. User Documentation

#### 3.1 Getting Started

```markdown
# docs/docs/getting-started/installation.md

# Installation

OSIRIS is a desktop application built with Tauri. Follow these steps to install:

## Prerequisites

- **Operating System**: macOS 11.0+, Windows 10+, or Linux (Ubuntu 18.04+)
- **RAM**: 4GB minimum, 8GB recommended
- **Disk Space**: 500MB for application, additional space for data

## Installation Methods

### macOS

#### Homebrew (Recommended)

```bash
brew install osiris
```

#### Manual Installation

1. Download the latest release from [GitHub Releases](https://github.com/osiris/osiris/releases)
2. Open the `.dmg` file
3. Drag OSIRIS to Applications folder
4. Open OSIRIS from Applications

### Windows

1. Download the latest `.exe` installer from [GitHub Releases](https://github.com/osiris/osiris/releases)
2. Run the installer
3. Follow the installation wizard
4. Launch OSIRIS from Start Menu

### Linux

#### AppImage

1. Download the latest `.AppImage` from [GitHub Releases](https://github.com/osiris/osiris/releases)
2. Make it executable: `chmod +x osiris.AppImage`
3. Run: `./osiris.AppImage`

#### Package Manager

```bash
# Debian/Ubuntu
sudo apt install osiris

# Fedora
sudo dnf install osiris

# Arch
yay -S osiris
```

## First Launch

1. Launch OSIRIS
2. The application will create a local database in your data directory
3. You'll see the main interface with an empty graph
4. Start by adding your first source (see [Adding Sources](/docs/user-guide/sources/overview))

## Verification

To verify installation:

1. Open OSIRIS
2. Go to Help → About
3. Check the version number matches the installed version

## Troubleshooting

### Application won't start

- Check system requirements
- Check logs in: `~/Library/Application Support/osiris/logs/` (macOS) or `%APPDATA%/osiris/logs/` (Windows)
- Try running from terminal to see error messages

### Database errors

- Delete the database file and restart (data will be lost)
- Database location: `~/Library/Application Support/osiris/osiris.db` (macOS)

## Next Steps

- [Quick Start Guide](/docs/getting-started/quick-start)
- [Configuration](/docs/getting-started/configuration)
- [User Guide](/docs/user-guide/overview)
```

#### 3.2 Feature Documentation

Every feature needs complete documentation:

```markdown
# docs/docs/user-guide/queries.md

# Queries

Queries allow you to search and filter entities in your OSINT graph.

## Query Basics

A query specifies what entities to find and how to filter them.

### Simple Query

Find all domains:

```rust
query {
    find: Domain
}
```

### Query with Filters

Find domains registered in the last 7 days:

```rust
query {
    find: Domain
    where: {
        registered_date > now() - 7 days
    }
}
```

## Query Language

### Entity Types

Specify which type of entity to find:

- `Domain` - Domain names
- `IP` - IP addresses
- `Email` - Email addresses
- `Person` - People
- `Twitter` - Twitter accounts
- `LinkedIn` - LinkedIn profiles
- `GitHub` - GitHub accounts
- `Article` - Articles from RSS feeds

### Filters

#### Comparison Filters

```rust
where: {
    domain = "example.com"           // Exact match
    domain != "example.com"          // Not equal
    confidence > 0.7                 // Greater than
    confidence >= 0.7                // Greater than or equal
    created_at < now() - 7 days      // Less than
    created_at <= now() - 30 days    // Less than or equal
}
```

#### String Filters

```rust
where: {
    name contains "John"             // Contains substring
    name starts_with "John"          // Starts with
    name ends_with "Doe"            // Ends with
    email matches "^[a-z]+@.*"       // Regex match
}
```

#### Relationship Filters

```rust
where: {
    related_to: {
        Domain: "example.com"
        relationship: UsesEmailDomain
    }
}
```

#### Source Filters

```rust
where: {
    sources: [WHOIS, Twitter, LinkedIn]
    source_count >= 2                // Verified by multiple sources
}
```

#### Confidence Filters

```rust
where: {
    confidence > 0.7                 // High confidence only
    confidence >= 0.5 && <= 0.8      // Medium confidence
}
```

### Expanding Relationships

Include related entities in results:

```rust
query {
    find: Domain
    where: { domain = "example.com" }
    expand: {
        relationships: all
        depth: 2                     // Expand 2 levels deep
    }
}
```

### Sorting

Sort results:

```rust
query {
    find: Domain
    sort_by: created_at
    order: descending
    limit: 100
}
```

## Query Examples

### Find All High-Confidence Entities

```rust
query {
    find: any
    where: { confidence > 0.8 }
    sort_by: confidence
    order: descending
}
```

### Find People Related to a Domain

```rust
query {
    find: Person
    where: {
        related_to: {
            Domain: "example.com"
        }
    }
    expand: {
        relationships: all
        depth: 1
    }
}
```

### Find Recent Twitter Accounts

```rust
query {
    find: Twitter
    where: {
        created_at > now() - 30 days
        sources: [Twitter]
    }
    sort_by: created_at
    order: descending
}
```

## Query Interface

### Using the Query Builder

1. Open the Query interface
2. Select entity type
3. Add filters using the UI
4. Click "Execute Query"
5. View results in the graph or table

### Using Query Language

1. Open the Query interface
2. Switch to "Query Language" mode
3. Type your query
4. Click "Execute"
5. View results

### Saving Queries

1. After executing a query, click "Save"
2. Give the query a name
3. Access saved queries from the sidebar

## Query Performance

- Simple queries: < 100ms
- Complex queries with relationships: < 500ms
- Large result sets: Use `limit` to restrict results

## See Also

- [Query Language Reference](/docs/reference/query-language)
- [Entity Types](/docs/reference/entity-types)
- [Relationships](/docs/user-guide/relationships)
```

---

### 4. Developer Documentation

#### 4.1 Architecture Documentation

```markdown
# docs/docs/developer-guide/architecture/overview.md

# Architecture Overview

OSIRIS follows a layered architecture with clear separation of concerns.

## Architecture Layers

```
┌─────────────────────────────────────┐
│         Presentation Layer          │
│    (Tauri Frontend + Leptos UI)     │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│         Application Layer           │
│    (Tauri Commands + Business Logic)│
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│          Domain Layer                │
│  (Entities, Relationships, Queries)│
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│        Infrastructure Layer          │
│  (Database, Sources, Monitoring)    │
└─────────────────────────────────────┘
```

## Key Components

### Presentation Layer

- **Tauri Frontend**: Native desktop window
- **Leptos UI**: Reactive UI components
- **Graph Visualization**: Interactive graph rendering
- **Query Interface**: Query builder and results display

### Application Layer

- **Tauri Commands**: API endpoints for frontend
- **Business Logic**: Core application logic
- **Query Execution**: Query parsing and execution
- **Relationship Detection**: Automatic relationship finding

### Domain Layer

- **Entity Model**: Core entity data structures
- **Relationship Model**: Relationship data structures
- **Query Model**: Query language and execution
- **Source Abstraction**: Unified source interface

### Infrastructure Layer

- **Database**: SQLite storage
- **Source Implementations**: RSS, WHOIS, Twitter, etc.
- **Monitoring System**: Real-time monitoring
- **Rate Limiting**: API rate limit management

## Data Flow

1. **User Action** → Frontend sends Tauri command
2. **Command Handler** → Processes request
3. **Business Logic** → Executes operation
4. **Domain Model** → Uses entities/relationships
5. **Infrastructure** → Accesses database/sources
6. **Response** → Returns data to frontend
7. **UI Update** → Frontend updates display

## Design Principles

- **Separation of Concerns**: Each layer has clear responsibilities
- **Dependency Inversion**: High-level modules don't depend on low-level
- **Single Responsibility**: Each module does one thing well
- **Open/Closed**: Open for extension, closed for modification

## See Also

- [Data Model](/docs/developer-guide/architecture/data-model)
- [Graph Structure](/docs/developer-guide/architecture/graph-structure)
- [Source Abstraction](/docs/developer-guide/architecture/source-abstraction)
```

#### 4.2 Extending Documentation

```markdown
# docs/docs/developer-guide/extending/adding-sources.md

# Adding a New Source

This guide explains how to add a new data source to OSIRIS.

## Source Interface

All sources implement the `Source` trait:

```rust
pub trait Source: Send + Sync {
    fn name(&self) -> &str;
    fn collect(&self, query: &Query) -> Result<Vec<Entity>>;
    fn monitor(&self, query: &Query) -> Result<Monitor>;
    fn rate_limit(&self) -> RateLimit;
    fn requires_auth(&self) -> bool;
    fn authenticate(&mut self, credentials: Credentials) -> Result<()>;
}
```

## Step-by-Step Guide

### 1. Create Source Struct

```rust
pub struct MySource {
    client: HttpClient,
    rate_limiter: RateLimiter,
    credentials: Option<Credentials>,
}
```

### 2. Implement Source Trait

```rust
impl Source for MySource {
    fn name(&self) -> &str {
        "MySource"
    }
    
    fn collect(&self, query: &Query) -> Result<Vec<Entity>> {
        self.rate_limiter.wait_if_needed()?;
        
        let response = self.client.get(&self.build_url(query))?;
        let data = response.json()?;
        
        let entities = self.extract_entities(&data)?;
        Ok(entities)
    }
    
    fn monitor(&self, query: &Query) -> Result<Monitor> {
        Ok(Monitor {
            source: SourceType::MySource,
            query: query.clone(),
            poll_interval: Duration::from_secs(300),
            // ...
        })
    }
    
    fn rate_limit(&self) -> RateLimit {
        RateLimit::per_minute(60)
    }
    
    fn requires_auth(&self) -> bool {
        true
    }
    
    fn authenticate(&mut self, credentials: Credentials) -> Result<()> {
        // Authenticate with source API
        self.credentials = Some(credentials);
        Ok(())
    }
}
```

### 3. Register Source

Add to source registry:

```rust
pub fn register_sources() -> Vec<Box<dyn Source>> {
    vec![
        Box::new(RSSSource::new()),
        Box::new(WHOISSource::new()),
        Box::new(MySource::new()), // Add your source
        // ...
    ]
}
```

### 4. Add UI Integration

Add source to UI source manager:

```rust
// In source manager component
<SourceTypeSelector>
    <Option value="RSS">RSS Feed</Option>
    <Option value="WHOIS">WHOIS</Option>
    <Option value="MySource">My Source</Option> // Add your source
</SourceTypeSelector>
```

### 5. Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_my_source_collection() {
        let source = MySource::new();
        let query = Query::new(EntityType::Domain);
        let entities = source.collect(&query).await.unwrap();
        assert!(!entities.is_empty());
    }
}
```

### 6. Document Source

Add documentation:

```markdown
# My Source

Description of the source and what data it provides.

## Configuration

- API Key: Required
- Rate Limit: 60 requests/minute

## Entity Types

- Domain
- IP

## See Also

- [Source API Reference](/docs/api-reference/sources/my-source)
```

## Best Practices

- Handle rate limiting properly
- Implement retry logic for transient errors
- Validate and sanitize input data
- Extract entities consistently
- Provide clear error messages
- Document rate limits and authentication

## See Also

- [Source Trait Reference](/docs/api-reference/sources/source-trait)
- [Entity Extraction](/docs/developer-guide/extending/entity-extraction)
- [Testing Sources](/docs/developer-guide/contributing/testing)
```

---

### 5. Documentation Generation

#### 5.1 Rust Doc Generation

```bash
# Generate API documentation
cargo doc --no-deps --open

# Generate with private items
cargo doc --document-private-items

# Generate for specific package
cargo doc -p osiris-core
```

#### 5.2 Documentation Scripts

```bash
#!/bin/bash
# scripts/generate-docs.sh

# Generate Rust API docs
echo "Generating Rust API documentation..."
cargo doc --no-deps

# Copy to Docusaurus
echo "Copying API docs to Docusaurus..."
cp -r target/doc/* docs/static/api-reference/

# Build Docusaurus
echo "Building Docusaurus site..."
cd docs
npm run build

echo "Documentation generated successfully!"
```

#### 5.3 CI/CD Documentation

```yaml
# .github/workflows/docs.yml
name: Documentation

on:
  push:
    branches: [main]
    paths:
      - 'src/**'
      - 'docs/**'

jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Generate API Docs
        run: cargo doc --no-deps
      
      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install Dependencies
        run: cd docs && npm install
      
      - name: Build Docusaurus
        run: cd docs && npm run build
      
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/build
```

---

### 6. Documentation Standards

#### 6.1 Code Examples

Every code example must:
- Be compilable (tested with `cargo test --doc`)
- Show complete usage
- Include error handling
- Be relevant to the topic

#### 6.2 Documentation Checklist

For every function/feature:
- [ ] Brief description
- [ ] Detailed explanation
- [ ] Parameter documentation
- [ ] Return value documentation
- [ ] Error documentation
- [ ] Code examples
- [ ] Related functions/types
- [ ] Performance notes (if relevant)

#### 6.3 User Documentation Checklist

For every user-facing feature:
- [ ] What it does
- [ ] How to use it
- [ ] Step-by-step guide
- [ ] Screenshots/diagrams
- [ ] Common use cases
- [ ] Troubleshooting
- [ ] Related features

---

### 7. Documentation Maintenance

#### 7.1 Documentation Reviews

- Review documentation with code reviews
- Update documentation when code changes
- Verify examples still work
- Check links are valid

#### 7.2 Documentation Testing

```rust
// Test that documentation examples compile
#[cfg(test)]
mod doc_tests {
    #[test]
    fn test_readme_example() {
        // Code from README should compile and run
    }
}
```

#### 7.3 Documentation Metrics

Track:
- Documentation coverage (% of functions documented)
- Broken links
- Outdated examples
- User feedback on documentation

---

## 🎉 Summary

**Complete Documentation System** includes:
- ✅ **Docusaurus Site**: Full documentation website
- ✅ **API Documentation**: Every function, struct, enum documented
- ✅ **User Documentation**: Complete user guides
- ✅ **Developer Documentation**: Architecture and extending guides
- ✅ **Tutorials**: Step-by-step guides
- ✅ **Examples**: Code examples for everything
- ✅ **Reference**: Complete reference documentation
- ✅ **Auto-Generation**: Rust doc + Docusaurus integration
- ✅ **CI/CD**: Automated documentation builds

**Result**: Comprehensive documentation ensuring users and developers can understand and use every aspect of OSIRIS.

---

## 🎉 Summary

**OSIRIS** is a novel OSINT platform that:
- Unifies all intelligence sources in one graph
- Works completely offline (privacy-first)
- Detects relationships automatically (no AI)
- Monitors sources in real-time
- Is open-source and accessible

**Perfect for doctorate**: Novel algorithms, practical application, publishable research, real-world impact.

