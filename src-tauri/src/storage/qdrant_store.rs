use anyhow::{Context, Result};
use qdrant_client::qdrant::{
    vectors_config::Config, CreateCollection, Distance, PointStruct,
    SearchPoints, VectorParams, VectorsConfig,
};
use qdrant_client::{Qdrant, config::QdrantConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDocument {
    pub id: String,
    pub collection: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
}

pub struct QdrantStore {
    client: Qdrant,
    default_collection: String,
}

impl QdrantStore {
    /// Create a new Qdrant store
    /// url: Qdrant server URL (default: http://localhost:6333)
    /// api_key: Optional API key for authentication
    pub async fn new(url: Option<&str>, api_key: Option<&str>) -> Result<Self> {
        let url = url.unwrap_or("http://localhost:6333");
        
        let mut config = QdrantConfig::from_url(url);
        if let Some(key) = api_key {
            config.api_key = Some(key.to_string());
        }
        
        let client = Qdrant::new(config)
            .context("Failed to create Qdrant client")?;
        
        Ok(QdrantStore {
            client,
            default_collection: "default".to_string(),
        })
    }
    
    /// Create a collection with specified dimension
    pub async fn create_collection(
        &self,
        name: &str,
        dimension: u64,
    ) -> Result<()> {
        let collection_config = CreateCollection {
            collection_name: name.to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: dimension,
                    distance: Distance::Cosine as i32,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };
        
        self.client
            .create_collection(collection_config)
            .await
            .context(format!("Failed to create collection: {}", name))?;
        
        Ok(())
    }
    
    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let collections = self.client
            .list_collections()
            .await
            .context("Failed to list collections")?;
        
        Ok(collections
            .collections
            .into_iter()
            .map(|c| c.name)
            .collect())
    }
    
    /// Insert a document into a collection
    pub async fn insert_document(
        &self,
        collection: &str,
        doc: &QdrantDocument,
    ) -> Result<()> {
        // Prepare payload (metadata)
        let mut payload: HashMap<String, qdrant_client::qdrant::Value> = HashMap::new();
        payload.insert(
            "content".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    doc.content.clone(),
                )),
            },
        );
        payload.insert(
            "collection".to_string(),
            qdrant_client::qdrant::Value {
                kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                    doc.collection.clone(),
                )),
            },
        );
        
        // Add custom metadata fields
        if let Some(obj) = doc.metadata.as_object() {
            for (key, value) in obj {
                let qdrant_value = match value {
                    serde_json::Value::String(s) => qdrant_client::qdrant::Value {
                        kind: Some(qdrant_client::qdrant::value::Kind::StringValue(
                            s.clone(),
                        )),
                    },
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            qdrant_client::qdrant::Value {
                                kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)),
                            }
                        } else if let Some(f) = n.as_f64() {
                            qdrant_client::qdrant::Value {
                                kind: Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)),
                            }
                        } else {
                            continue;
                        }
                    }
                    serde_json::Value::Bool(b) => qdrant_client::qdrant::Value {
                        kind: Some(qdrant_client::qdrant::value::Kind::BoolValue(*b)),
                    },
                    _ => continue,
                };
                payload.insert(format!("metadata_{}", key), qdrant_value);
            }
        }
        
        // Create point
        let point = PointStruct::new(
            doc.id.clone(),
            doc.embedding.clone(),
            payload,
        );
        
        // Upsert point - new API uses a single request struct
        use qdrant_client::qdrant::UpsertPoints;
        let upsert_request = UpsertPoints {
            collection_name: collection.to_string(),
            points: vec![point],
            ..Default::default()
        };
        self.client
            .upsert_points(upsert_request)
            .await
            .context(format!("Failed to insert document into collection: {}", collection))?;
        
        Ok(())
    }
    
    /// Search for similar documents
    pub async fn search_similar(
        &self,
        collection: &str,
        query_embedding: &[f32],
        limit: u64,
        min_score: Option<f32>,
    ) -> Result<Vec<(QdrantDocument, f32)>> {
        let search_request = SearchPoints {
            collection_name: collection.to_string(),
            vector: query_embedding.to_vec(),
            limit,
            score_threshold: min_score,
            with_payload: Some(true.into()),
            ..Default::default()
        };
        
        let search_result = self.client
            .search_points(search_request)
            .await
            .context(format!("Failed to search collection: {}", collection))?;
        
        let mut results = Vec::new();
        for result in search_result.result {
            let score = result.score;
            
            // Extract payload
            let payload = result.payload;
            let content = payload
                .get("content")
                .and_then(|v| {
                    if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &v.kind {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            
            let collection_name = payload
                .get("collection")
                .and_then(|v| {
                    if let Some(qdrant_client::qdrant::value::Kind::StringValue(s)) = &v.kind {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| collection.to_string());
            
            // Reconstruct metadata from payload
            let mut metadata = serde_json::json!({});
            for (key, value) in &payload {
                if key.starts_with("metadata_") {
                    let meta_key = key.strip_prefix("metadata_").unwrap_or(key);
                    if let Some(kind) = &value.kind {
                        match kind {
                            qdrant_client::qdrant::value::Kind::StringValue(s) => {
                                metadata[meta_key] = serde_json::json!(s);
                            }
                            qdrant_client::qdrant::value::Kind::IntegerValue(i) => {
                                metadata[meta_key] = serde_json::json!(i);
                            }
                            qdrant_client::qdrant::value::Kind::DoubleValue(f) => {
                                metadata[meta_key] = serde_json::json!(f);
                            }
                            qdrant_client::qdrant::value::Kind::BoolValue(b) => {
                                metadata[meta_key] = serde_json::json!(b);
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // Get vector from result if available
            let embedding = if let Some(vectors) = result.vectors {
                // Extract vector data from the Vectors enum
                // The structure depends on the qdrant-client version
                // For now, try to extract the first vector if available
                vec![] // Will be populated from payload or result if needed
            } else {
                vec![]
            };
            
            // Extract ID from PointId - convert to string representation
            let point_id_str = if let Some(id) = &result.id {
                // Try to extract numeric or UUID ID
                format!("{:?}", id)
                    .trim_start_matches("PointId { point_id_options: Some(")
                    .trim_end_matches(") }")
                    .to_string()
            } else {
                "unknown".to_string()
            };
            
            let doc = QdrantDocument {
                id: point_id_str,
                collection: collection_name,
                content,
                embedding,
                metadata,
            };
            
            results.push((doc, score));
        }
        
        Ok(results)
    }
    
    /// Delete a collection
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client
            .delete_collection(name)
            .await
            .context(format!("Failed to delete collection: {}", name))?;
        Ok(())
    }
    
    /// Get collection info
    pub async fn get_collection_info(&self, name: &str) -> Result<CollectionInfo> {
        let info = self.client
            .collection_info(name)
            .await
            .context(format!("Failed to get collection info: {}", name))?;
        
        // Extract info from the response
        let points_count = info.result
            .as_ref()
            .and_then(|r| r.points_count)
            .unwrap_or(0);
        // vectors_count field doesn't exist in new API, use points_count as approximation
        let vectors_count = points_count;
        
        Ok(CollectionInfo {
            name: name.to_string(),
            points_count,
            vectors_count,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub points_count: u64,
    pub vectors_count: u64,
}
