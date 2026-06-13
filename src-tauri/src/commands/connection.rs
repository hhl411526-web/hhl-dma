use tauri::State;
use std::sync::Arc;
use crate::core::connection::manager::ConnectionManager;
use crate::core::types::{ConnectionConfig, DatabaseType};
use crate::errors::AppError;
use chrono::Utc;

#[tauri::command]
pub async fn create_connection(
    manager: State<'_, Arc<ConnectionManager>>,
    name: String,
    db_type: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    schema: Option<String>,
    group: Option<String>,
) -> Result<String, AppError> {
    let db_type = match db_type.as_str() {
        "mysql" => DatabaseType::MySQL,
        "postgresql" => DatabaseType::PostgreSQL,
        "sqlite" => DatabaseType::SQLite,
        "oracle" => DatabaseType::Oracle,
        "sqlserver" => DatabaseType::SQLServer,
        "dm" => DatabaseType::DM,
        "kingbasees" => DatabaseType::KingbaseES,
        "odbc" => DatabaseType::ODBC,
        _ => return Err(AppError::Connection(format!("Unsupported database type: {}", db_type))),
    };
    let config = ConnectionConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        db_type,
        host,
        port,
        username,
        password,
        database,
        schema,
        options: std::collections::HashMap::new(),
        group,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    manager.create_connection(&config).await
}

#[tauri::command]
pub async fn close_connection(
    manager: State<'_, Arc<ConnectionManager>>,
    conn_id: String,
) -> Result<(), AppError> {
    manager.close_connection(&conn_id).await
}

#[tauri::command]
pub async fn test_connection(
    manager: State<'_, Arc<ConnectionManager>>,
    db_type: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
) -> Result<bool, AppError> {
    let db_type = match db_type.as_str() {
        "mysql" => DatabaseType::MySQL,
        "postgresql" => DatabaseType::PostgreSQL,
        "sqlite" => DatabaseType::SQLite,
        _ => return Err(AppError::Connection(format!("Unsupported database type: {}", db_type))),
    };
    let config = ConnectionConfig {
        id: String::new(),
        name: String::new(),
        db_type,
        host,
        port,
        username,
        password,
        database,
        schema: None,
        options: std::collections::HashMap::new(),
        group: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    manager.test_connection(&config).await
}

#[tauri::command]
pub async fn list_saved_connections(
    manager: State<'_, Arc<ConnectionManager>>,
) -> Result<Vec<ConnectionConfig>, AppError> {
    manager.load_saved_connections().await
}

#[tauri::command]
pub async fn delete_connection(
    manager: State<'_, Arc<ConnectionManager>>,
    id: String,
) -> Result<(), AppError> {
    manager.delete_connection_config(&id).await
}
