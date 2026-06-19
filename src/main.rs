// =============================================
// ТОЧКА ВХОДА В ПРИЛОЖЕНИЕ.
// =============================================

mod config;
mod qdrant;
mod ollama;
mod indexer;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 MP Indexer starting...\n");
    
    // Загружаем конфигурацию
    let config = config::Config::from_env()?;
    println!("✅ Config loaded");
    println!("   📁 Data path: {}", config.data_path);
    println!("   🧠 Model: {}", config.embedding_model);
    println!("   📐 Vector size: {}", config.vector_size);
    println!("   📏 Chunk size: {}", config.chunk_size);
    
    // Проверяем Ollama
    println!("\n🔍 Checking services...");
    if let Err(e) = ollama::check_ollama_health(&config).await {
        eprintln!("⚠️ Warning: {}", e);
        eprintln!("   Ollama may not be running. Continuing anyway...");
    }
    
    // Создаем клиенты
    println!("\n🗄️ Connecting to services...");
    let qdrant = qdrant::QdrantClient::new(&config).await?;
    let ollama = ollama::OllamaClient::new(&config);
    
    // Запускаем индексацию
    println!("\n📄 Starting indexing...\n");
    let indexer = indexer::Indexer::new(config, qdrant, ollama);
    indexer.index_all().await?;
    
    println!("\n✅ Done!");
    Ok(())
}