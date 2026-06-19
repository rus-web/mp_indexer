// =============================================
// МОДУЛЬ РАБОТЫ С QDRANT.
// =============================================

use std::collections::HashMap;

use anyhow::{Context, Result};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder,
    VectorParamsBuilder,
    Distance,
    PointStruct,
    UpsertPointsBuilder,
};
use serde_json::Value;
use crate::config::Config;

/// Клиент для работы с Qdrant
pub struct QdrantClient {
    client: Qdrant,
}

impl QdrantClient {
    /// Создает новый клиент Qdrant
    pub async fn new(config: &Config) -> Result<Self> {
        println!("🗃️ Connecting to Qdrant: {}", config.qdrant_url);
        
        let client = Qdrant::from_url(&config.qdrant_url)
            .api_key(config.qdrant_api_key.clone())
            .build()
            .context("Failed to build Qdrant client")?;
        
        Ok(Self { client })
    }

    /// Создает коллекцию, если она не существует
    pub async fn ensure_collection(&self, name: &str, vector_size: u64) -> Result<()> {
        let exists = self.client
            .collection_exists(name)
            .await
            .context(format!("Failed to check collection '{}'", name))?;
        
        if !exists {
            println!("🗄️ Creating collection: {}", name);
            
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(name)
                        .vectors_config(
                            VectorParamsBuilder::new(vector_size, Distance::Cosine)
                        )
                )
                .await
                .context(format!("Failed to create collection '{}'", name))?;
            
            println!("✅ Collection '{}' created", name);
        } else {
            println!("✅ Collection '{}' already exists", name);
        }
        
        Ok(())
    }

    /// Сохраняет точки в коллекцию
    pub async fn upsert_points(&self, name: &str, points: Vec<PointStruct>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }
        
        println!("💾 Saving {} points to '{}'", points.len(), name);
        
        self.client
            .upsert_points(UpsertPointsBuilder::new(name, points))
            .await
            .context(format!("Failed to upsert points to '{}'", name))?;
        
        println!("✅ Points saved");
        Ok(())
    }
}



pub fn create_point(id: &str, vector: Vec<f32>, payload: Value) -> PointStruct {
    let payload_map: HashMap<String, serde_json::Value> = match payload {
        Value::Object(map) => {
            map.into_iter()
                .map(|(k, v)| (k, v))
                .collect()
        }
        _ => HashMap::new(),        
    };

    PointStruct::new(
        id.to_string(),
        vector,
        payload_map,  // HashMap<String, serde_json::Value> реализует Into<Payload>
    )
}


