use crate::providers::ollama::{OllamaProvider, OllamaModel, ChatMessage};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};

// Global Ollama provider state
pub type OllamaState = Arc<RwLock<OllamaProvider>>;

#[tauri::command]
pub async fn check_ollama_status(
    ollama: State<'_, OllamaState>,
) -> Result<bool, String> {
    let provider = ollama.read().await;
    provider.check_ollama_running().await
        .map_err(|e| format!("Failed to check Ollama status: {}", e))
}

#[tauri::command]
pub async fn list_ollama_models(
    ollama: State<'_, OllamaState>,
) -> Result<Vec<OllamaModel>, String> {
    let provider = ollama.read().await;
    provider.list_models().await
        .map_err(|e| format!("Failed to list models: {}", e))
}

#[tauri::command]
pub async fn get_ollama_model_info(
    model_name: String,
    ollama: State<'_, OllamaState>,
) -> Result<serde_json::Value, String> {
    let provider = ollama.read().await;
    let info = provider.get_model_info(&model_name).await
        .map_err(|e| format!("Failed to get model info: {}", e))?;
    
    serde_json::to_value(info)
        .map_err(|e| format!("Failed to serialize model info: {}", e))
}

#[tauri::command]
pub async fn load_model_from_file(
    file_path: String,
    ollama: State<'_, OllamaState>,
) -> Result<String, String> {
    let provider = ollama.read().await;
    let path = PathBuf::from(file_path);
    provider.load_model_from_file(&path).await
        .map_err(|e| format!("Failed to load model: {}", e))
}

#[tauri::command]
pub async fn chat_with_ollama(
    model: String,
    messages: Vec<ChatMessage>,
    ollama: State<'_, OllamaState>,
) -> Result<String, String> {
    let provider = ollama.read().await;
    provider.chat(&model, messages).await
        .map_err(|e| format!("Failed to chat with Ollama: {}", e))
}

#[tauri::command]
pub async fn scan_models_folder(
    ollama: State<'_, OllamaState>,
) -> Result<Vec<String>, String> {
    let provider = ollama.read().await;
    let models = provider.scan_models_folder().await
        .map_err(|e| format!("Failed to scan models folder: {}", e))?;
    
    Ok(models
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
pub async fn get_models_folder_path(
    ollama: State<'_, OllamaState>,
) -> Result<String, String> {
    let provider = ollama.read().await;
    Ok(provider.get_models_folder().to_string_lossy().to_string())
}

// Start watching the models folder for new files
pub async fn start_models_folder_watcher(
    ollama: Arc<RwLock<OllamaProvider>>,
) -> Result<RecommendedWatcher, String> {
    // Get the models folder path
    let models_folder = {
        let provider = ollama.read().await;
        provider.get_models_folder().to_path_buf()
    };
    
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                if let EventKind::Create(_) | EventKind::Modify(_) = event.kind {
                    // New file added or modified
                    if let Some(path) = event.paths.first() {
                        // Check if it's a model file
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if matches!(ext_str.as_str(), "gguf" | "bin" | "safetensors" | "pt" | "pth") {
                                println!("New model file detected: {:?}", path);
                                // In a real implementation, you might want to emit a Tauri event here
                                // or trigger model loading automatically
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    })
    .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    watcher
        .watch(&models_folder, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch models folder: {}", e))?;

    Ok(watcher)
}
