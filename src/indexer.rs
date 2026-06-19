// =============================================
// МОДУЛЬ ИНДЕКСАЦИИ.
// =============================================

use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;
use uuid::Uuid;
use serde_json::json;
use colored::*;  // Добавить в Cargo.toml

use crate::config::Config;
use crate::qdrant::{QdrantClient, create_point};
use crate::ollama::OllamaClient;

pub struct Indexer {
    config: Config,
    qdrant: QdrantClient,
    ollama: OllamaClient,
    chunk_size: usize,
}

impl Indexer {
    pub fn new(config: Config, qdrant: QdrantClient, ollama: OllamaClient) -> Self {
        let chunk_size = config.chunk_size;  
        Self {
            config,      
            qdrant,
            ollama,
            chunk_size,  
        }
    }

    pub async fn index_all(&self) -> Result<()> {
        println!("{}", "╔════════════════════════════════════════╗".bright_green());
        println!("{}", "║     MP-RAG System - Indexer v1.0       ║".bright_green());
        println!("{}", "╚════════════════════════════════════════╝".bright_green());

        let data_path = Path::new(&self.config.data_path);
        
        if !data_path.exists() {
            std::fs::create_dir_all(data_path)
                .context("Failed to create data directory")?;
            println!("📁 Created data directory: {:?}", data_path);
            println!("{}", "   ⚠️  Please add files to the data/ directory".yellow());
            return Ok(());
        }

        // Сканируем подпапки
        for entry in std::fs::read_dir(data_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let collection_name = path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                
                self.index_collection(&path, &collection_name).await?;
            }
        }

        Ok(())
    }

    async fn index_collection(&self, path: &Path, collection_name: &str) -> Result<()> {
        println!("\n📂 Indexing collection: {}", collection_name);
        
        self.qdrant.ensure_collection(collection_name, self.config.vector_size).await?;
        
        let mut points = Vec::new();
        let supported_exts = [
            ".txt", ".md", ".rs", ".py", ".js", 
            ".json", ".csv", ".toml", ".yaml", ".yml",
        ];
        
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            
            if !file_path.is_file() {
                continue;
            }
            
            let ext = file_path.extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e.to_lowercase()))
                .unwrap_or_default();
            
            if !supported_exts.contains(&ext.as_str()) {
                continue;
            }
            
            let text = std::fs::read_to_string(file_path)?;
            if text.trim().is_empty() {
                continue;
            }
            
            println!("   📄 Processing: {}", file_path.display());
            
            // Используем вашу умную функцию чанкинга
            let chunks = self.chunk_text(&text);
            let file_name = file_path.file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            
            println!("   📊 Created {} chunks", chunks.len());
            
            for (i, chunk) in chunks.iter().enumerate() {
                print!("\r   🔄 Generating embedding for chunk {}/{}", i + 1, chunks.len());
                
                let embedding = self.ollama.embed(chunk).await?;
                
                let payload = json!({
                    "text": chunk,
                    "collection": collection_name,
                    "file_path": file_path.to_string_lossy(),
                    "file_name": file_name,
                    "chunk_index": i,
                    "total_chunks": chunks.len(),
                    "file_extension": ext,
                });
                
                let point = create_point(
                    &Uuid::new_v4().to_string(),
                    embedding,
                    payload,
                );
                points.push(point);
                
                // Сохраняем каждые 50 чанков
                if points.len() >= 50 {
                    self.qdrant.upsert_points(collection_name, points).await?;
                    points = Vec::new();
                }
            }
            
            println!("\n   ✅ Done processing: {}", file_name);
        }
        
        if !points.is_empty() {
            self.qdrant.upsert_points(collection_name, points).await?;
        }
        
        println!("✅ Collection '{}' indexed", collection_name);
        Ok(())
    }

    /// Умная функция разбиения текста на чанки (из вашего кода)
    fn chunk_text(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;
        
        while start < chars.len() {
            let mut end = (start + self.chunk_size).min(chars.len());
            
            // Стараемся разорвать на границе предложения или пробела
            if end < chars.len() {
                let mut new_end = end;
                for i in (start..end).rev() {
                    if chars[i] == ' ' || chars[i] == '.' || chars[i] == '!' || 
                       chars[i] == '?' || chars[i] == '\n' {
                        new_end = i + 1;
                        break;
                    }
                }
                end = new_end;
            }
            
            let chunk: String = chars[start..end].iter().collect();
            if !chunk.trim().is_empty() {
                chunks.push(chunk);
            }
            
            start = end;
        }
        
        chunks
    }
}