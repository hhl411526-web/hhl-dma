use std::sync::Arc;
use crate::core::connection::pool::ConnectionPool;
use crate::core::connection::config::ConnectionConfigStore;
use crate::core::plugin::registry::PluginRegistry;
use crate::core::types::ConnectionConfig;
use crate::errors::AppError;

pub struct ConnectionManager {
    pool: Arc<ConnectionPool>,
    config_store: Arc<ConnectionConfigStore>,
    plugin_registry: Arc<PluginRegistry>,
}

impl ConnectionManager {
    pub fn new(
        pool: Arc<ConnectionPool>,
        config_store: Arc<ConnectionConfigStore>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self { pool, config_store, plugin_registry }
    }

    pub async fn create_connection(&self, config: &ConnectionConfig) -> Result<String, AppError> {
        let driver = self.plugin_registry.get_driver(&config.db_type).await?;
        let conn_id = driver.connect(config).await?;
        self.pool.register(conn_id.clone(), config.clone()).await;
        self.config_store.save(config).await?;
        Ok(conn_id)
    }

    pub async fn close_connection(&self, conn_id: &str) -> Result<(), AppError> {
        let config = self.pool.get_config(conn_id).await?;
        let driver = self.plugin_registry.get_driver(&config.db_type).await?;
        driver.disconnect(conn_id).await?;
        self.pool.unregister(conn_id).await?;
        Ok(())
    }

    pub async fn test_connection(&self, config: &ConnectionConfig) -> Result<bool, AppError> {
        let driver = self.plugin_registry.get_driver(&config.db_type).await?;
        driver.test_connection(config).await
    }

    pub async fn load_saved_connections(&self) -> Result<Vec<ConnectionConfig>, AppError> {
        self.config_store.load_all().await
    }

    pub async fn delete_connection_config(&self, id: &str) -> Result<(), AppError> {
        self.config_store.delete(id).await
    }

    pub fn pool(&self) -> Arc<ConnectionPool> {
        self.pool.clone()
    }
}
