use crate::services::embeddings::EmbeddingService;
use crate::services::api_key_manager::APIKeyManager;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn generate_embedding(
    text: String,
    _dimension: Option<usize>,
    api_key_manager: State<'_, Arc<APIKeyManager>>,
) -> Result<Vec<f32>, String> {
    let mut service = EmbeddingService::new();
    
    // Get OpenAI API key from API key manager if available
    if let Ok(Some(openai_key)) = api_key_manager.get_key_optional("openai") {
        service.set_openai_key(openai_key);
    }
    
    let embedding = service.generate(&text).await
        .map_err(|e| format!("Failed to generate embedding: {}", e))?;
    
    Ok(embedding)
}

