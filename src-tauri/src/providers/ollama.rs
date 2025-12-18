use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub size: u64,
    pub digest: String,
    pub details: ModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: Option<ChatMessageResponse>,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    role: String,
    content: String,
}

pub struct OllamaProvider {
    base_url: String,
    models_folder: PathBuf,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(models_folder: PathBuf) -> Self {
        Self {
            base_url: OLLAMA_BASE_URL.to_string(),
            models_folder,
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_ollama_running(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API returned error: {}", response.status());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let models = json
            .get("models")
            .and_then(|m| m.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        let mut result = Vec::new();
        for model in models {
            result.push(OllamaModel {
                name: model
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                size: model
                    .get("size")
                    .and_then(|s| s.as_u64())
                    .unwrap_or(0),
                modified_at: model
                    .get("modified_at")
                    .and_then(|m| m.as_str())
                    .unwrap_or("")
                    .to_string(),
                digest: model
                    .get("digest")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }

        Ok(result)
    }

    pub async fn get_model_info(&self, model_name: &str) -> Result<OllamaModelInfo> {
        let url = format!("{}/api/show", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": model_name }))
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API returned error: {}", response.status());
        }

        let info: OllamaModelInfo = response
            .json()
            .await
            .context("Failed to parse model info")?;

        Ok(info)
    }

    pub async fn load_model_from_file(&self, model_path: &Path) -> Result<String> {
        // For HuggingFace models, we need to import them into Ollama
        // This requires the model to be in GGUF format or compatible format
        // We'll use Ollama's import/create API
        
        let model_name = model_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?;

        // Check if model already exists
        let existing_models = self.list_models().await.unwrap_or_default();
        if existing_models.iter().any(|m| m.name == model_name) {
            return Ok(format!("Model '{}' already loaded", model_name));
        }

        // For now, we'll return instructions since direct file import requires
        // specific Ollama setup. In production, you'd use:
        // ollama create <model_name> -f <modelfile_path>
        // or use Ollama's import API if the model is in the right format
        
        Ok(format!(
            "Model file detected: {}. To load it into Ollama, ensure it's in GGUF format and use: ollama create {} -f <path_to_modelfile>",
            model_path.display(),
            model_name
        ))
    }

    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages: messages.clone(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send chat request to Ollama")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error: {} - {}", status, error_text);
        }

        // Parse the response
        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama chat response")?;

        Ok(chat_response
            .message
            .map(|m| m.content)
            .unwrap_or_else(|| "No response from model".to_string()))
    }


    pub fn get_models_folder(&self) -> &Path {
        &self.models_folder
    }

    pub async fn scan_models_folder(&self) -> Result<Vec<PathBuf>> {
        let mut models = Vec::new();
        
        if !self.models_folder.exists() {
            std::fs::create_dir_all(&self.models_folder)
                .context("Failed to create models folder")?;
            return Ok(models);
        }

        let entries = std::fs::read_dir(&self.models_folder)
            .context("Failed to read models folder")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            // Check for common model file extensions
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(ext_str.as_str(), "gguf" | "bin" | "safetensors" | "pt" | "pth") {
                    models.push(path);
                }
            }
        }

        Ok(models)
    }
}

