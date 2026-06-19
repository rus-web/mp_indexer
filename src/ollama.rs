// =============================================
// МОДУЛЬ РАБОТЫ С OLLAMA.
// Отправка текста в Ollama и получение эмбеддингов.
// =============================================

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::Config;

// Запрос к Ollama API для получения эмбеддинга
#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    input: String,
}

// Ответ от Ollama API
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

// Клиент для работы с Ollama
pub struct OllamaClient {
    client: Client,
    url: String,
    model: String,
}

impl OllamaClient {
    /// Создает новый клиент Ollama
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            url: config.ollama_url.clone(),
            model: config.embedding_model.clone(),
        }
    }
    
    /// Получает эмбеддинг для текста
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbedRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };
        
        let response = self.client
            .post(&format!("{}/api/embed", self.url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama returned error {}: {}",
                status,
                text
            ));
        }
        
        let embed_response: EmbedResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;
        
        embed_response
            .embeddings
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned from Ollama"))
    }
}

/// Проверяет доступность Ollama
pub async fn check_ollama_health(config: &Config) -> Result<()> {
    let client = Client::new();
    let response = client
        .get(&format!("{}/api/tags", config.ollama_url))
        .send()
        .await
        .context("Failed to connect to Ollama")?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Ollama returned error: {}",
            response.status()
        ));
    }
    
    println!("✅ Ollama is available at {}", config.ollama_url);
    println!("   Using model: {}", config.embedding_model);
    Ok(())
}