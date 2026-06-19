// =============================================
// МОДУЛЬ КОНФИГУРАЦИИ.
// Загружает настройки из переменных окружения (.env).
// =============================================

use std::env;
use anyhow::{Result, Context};
use dotenv::dotenv;

// Структура конфигурации приложения.
#[derive(Debug, Clone)]
pub struct  Config {
    pub qdrant_url: String,
    pub qdrant_api_key: String,
    pub ollama_url: String,
    pub embedding_model: String,
    pub data_path: String,
    pub vector_size: u64,
    pub chunk_size: usize,
}

impl Config {
    /// Загружает конфигурацию из .env
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        
        Ok(Config {
            qdrant_url: env::var("QDRANT_URL")
                .context("QDRANT_URL undefined in .env")?,
            
            qdrant_api_key: env::var("QDRANT_INDEXER_KEY")
                .context("QDRANT_INDEXER_KEY undefined in .env")?,
            
            ollama_url: env::var("OLLAMA_URL")
                .context("OLLAMA_URL undefined in .env")?,
            
            embedding_model: env::var("EMBEDDING_MODEL")
                .context("EMBEDDING_MODEL undefined in .env")?,
            
            data_path: env::var("DATA_PATH")
                .unwrap_or_else(|_| "./data".to_string()),
            
            vector_size: env::var("VECTOR_SIZE")
                .context("VECTOR_SIZE undefined in .env")?
                .parse()
                .context("VECTOR_SIZE undefined in .env")?,
            chunk_size: env::var("CHUNK_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .context("CHUNK_SIZE must be a valid number")?,
        })
    }
}

