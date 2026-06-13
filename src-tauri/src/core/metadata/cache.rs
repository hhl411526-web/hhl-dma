use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::core::types::DatabaseMetadata;

pub struct MetadataCache {
    cache: RwLock<HashMap<String, CachedMetadata>>,
    ttl_secs: u64,
}

struct CachedMetadata {
    metadata: DatabaseMetadata,
    cached_at: chrono::DateTime<chrono::Utc>,
}

impl MetadataCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl_secs,
        }
    }

    pub async fn get(&self, conn_id: &str) -> Option<DatabaseMetadata> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(conn_id) {
            let now = chrono::Utc::now();
            let age = now - cached.cached_at;
            if age.num_seconds() < self.ttl_secs as i64 {
                return Some(cached.metadata.clone());
            }
        }
        None
    }

    pub async fn put(&self, conn_id: &str, metadata: DatabaseMetadata) {
        let mut cache = self.cache.write().await;
        cache.insert(conn_id.to_string(), CachedMetadata {
            metadata,
            cached_at: chrono::Utc::now(),
        });
    }

    pub async fn invalidate(&self, conn_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(conn_id);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}
