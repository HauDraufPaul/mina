use crate::services::embeddings::EmbeddingService;
use std::sync::Mutex;
use tauri::State;

// Global embedding service instance (will be managed by Tauri state)
// For now, we'll create a new instance each time, but this should be managed state
// TODO: Move to managed state once API key management is implemented

#[tauri::command]
pub async fn generate_embedding(text: String, dimension: Option<usize>) -> Result<Vec<f32>, String> {
    let mut service = EmbeddingService::new();
    
    // TODO: Get API key from API key manager when Phase 2.1 is implemented
    // For now, service will use local fallback
    
    let embedding = service.generate(&text).await
        .map_err(|e| format!("Failed to generate embedding: {}", e))?;
    
    Ok(embedding)
}

