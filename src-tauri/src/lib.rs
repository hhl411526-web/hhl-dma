mod core;
mod commands;
mod errors;

use std::sync::Arc;
use core::connection::manager::ConnectionManager;
use core::connection::pool::ConnectionPool;
use core::connection::config::ConnectionConfigStore;
use core::executor::engine::SqlEngine;
use core::executor::history::ExecutionHistory;
use core::metadata::cache::MetadataCache;
use core::plugin::registry::PluginRegistry;
use core::plugin::mysql_driver::MysqlDriver;
use core::plugin::postgresql_driver::PostgresqlDriver;
use core::plugin::sqlite_driver::SqliteDriver;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let plugin_registry = Arc::new(PluginRegistry::new());
    let pool = Arc::new(ConnectionPool::new());
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("hhl-dma")
        .join("connections");
    let config_store = Arc::new(ConnectionConfigStore::new(config_dir));
    let connection_manager = Arc::new(ConnectionManager::new(
        pool,
        config_store,
        plugin_registry.clone(),
    ));
    let history = Arc::new(ExecutionHistory::new(1000));
    let engine = Arc::new(SqlEngine::new(connection_manager.clone(), plugin_registry.clone(), history));
    let metadata_cache = Arc::new(MetadataCache::new(300));

    // Register MySQL driver
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mysql_driver = Arc::new(MysqlDriver::new());
        plugin_registry.register(mysql_driver).await.unwrap();

        let pg_driver = Arc::new(PostgresqlDriver::new());
        plugin_registry.register(pg_driver).await.unwrap();

        let sqlite_driver = Arc::new(SqliteDriver::new());
        plugin_registry.register(sqlite_driver).await.unwrap();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(connection_manager)
        .manage(engine)
        .manage(metadata_cache)
        .manage(plugin_registry)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::connection::create_connection,
            commands::connection::close_connection,
            commands::connection::test_connection,
            commands::connection::list_saved_connections,
            commands::connection::delete_connection,
            commands::query::execute_query,
            commands::query::execute_query_paginated,
            commands::query::cancel_query,
            commands::query::get_query_history,
            commands::metadata::get_database_metadata,
            commands::metadata::get_table_schema,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
