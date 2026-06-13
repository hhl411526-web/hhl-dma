use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::core::types::ConnectionConfig;
use crate::errors::AppError;

struct PoolEntry {
    config: ConnectionConfig,
    pool_size: usize,
    max_size: usize,
    idle_since: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ConnectionPool {
    pools: RwLock<HashMap<String, PoolEntry>>,
    default_max_pool_size: usize,
    idle_timeout_secs: u64,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            default_max_pool_size: 5,
            idle_timeout_secs: 300,
        }
    }

    pub async fn register(&self, conn_id: String, config: ConnectionConfig) {
        let mut pools = self.pools.write().await;
        pools.insert(conn_id, PoolEntry {
            config,
            pool_size: 1,
            max_size: self.default_max_pool_size,
            idle_since: None,
        });
    }

    pub async fn unregister(&self, conn_id: &str) -> Result<ConnectionConfig, AppError> {
        let mut pools = self.pools.write().await;
        pools.remove(conn_id)
            .map(|e| e.config)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found in pool", conn_id)))
    }

    pub async fn get_config(&self, conn_id: &str) -> Result<ConnectionConfig, AppError> {
        let pools = self.pools.read().await;
        pools.get(conn_id)
            .map(|e| e.config.clone())
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found in pool", conn_id)))
    }

    pub async fn list_active(&self) -> Vec<(String, String)> {
        let pools = self.pools.read().await;
        pools.iter()
            .map(|(id, entry)| (id.clone(), entry.config.name.clone()))
            .collect()
    }

    pub async fn cleanup_idle(&self) -> Vec<String> {
        let mut pools = self.pools.write().await;
        let now = chrono::Utc::now();
        let timeout = chrono::Duration::seconds(self.idle_timeout_secs as i64);
        let mut removed = Vec::new();
        pools.retain(|id, entry| {
            if let Some(idle_since) = entry.idle_since {
                if now - idle_since > timeout {
                    removed.push(id.clone());
                    return false;
                }
            }
            true
        });
        removed
    }
}
