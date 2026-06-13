use tauri::State;
use std::sync::Arc;
use crate::core::connection::manager::ConnectionManager;
use crate::core::metadata::cache::MetadataCache;
use crate::core::plugin::registry::PluginRegistry;
use crate::core::types::{DatabaseMetadata, TableSchema};
use crate::errors::AppError;

#[tauri::command]
pub async fn get_database_metadata(
    manager: State<'_, Arc<ConnectionManager>>,
    cache: State<'_, Arc<MetadataCache>>,
    registry: State<'_, Arc<PluginRegistry>>,
    conn_id: String,
    refresh: Option<bool>,
) -> Result<DatabaseMetadata, AppError> {
    if refresh != Some(true) {
        if let Some(metadata) = cache.get(&conn_id).await {
            return Ok(metadata);
        }
    }
    let config = manager.pool().get_config(&conn_id).await?;
    let driver = registry.get_driver(&config.db_type).await?;
    let metadata = driver.metadata(&conn_id).await?;
    cache.put(&conn_id, metadata.clone()).await;
    Ok(metadata)
}

#[tauri::command]
pub async fn get_table_schema(
    manager: State<'_, Arc<ConnectionManager>>,
    registry: State<'_, Arc<PluginRegistry>>,
    conn_id: String,
    table: String,
) -> Result<TableSchema, AppError> {
    let config = manager.pool().get_config(&conn_id).await?;
    let driver = registry.get_driver(&config.db_type).await?;
    driver.table_schema(&conn_id, &table).await
}
