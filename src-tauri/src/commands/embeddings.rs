use crate::utils::embeddings::EmbeddingGenerator;

#[tauri::command]
pub fn generate_embedding(text: String, dimension: Option<usize>) -> Result<Vec<f32>, String> {
    let dim = dimension.unwrap_or(384);
    let generator = EmbeddingGenerator::new(dim);
    Ok(generator.generate_weighted(&text))
}

