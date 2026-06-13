use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Row, Column};
use crate::core::plugin::driver::DatabaseDriver;
use crate::core::types::*;
use crate::errors::AppError;

pub struct MysqlDriver {
    connections: RwLock<HashMap<String, sqlx::Pool<MySql>>>,
}

impl MysqlDriver {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    fn build_connection_url(config: &ConnectionConfig) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username,
            urlencoding::encode(&config.password),
            config.host,
            config.port,
            config.database
        )
    }
}

#[async_trait]
impl DatabaseDriver for MysqlDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "MySQL Driver".to_string(),
            version: "0.1.0".to_string(),
            db_type: DatabaseType::MySQL,
            description: "MySQL/MariaDB database driver".to_string(),
        }
    }

    async fn connect(&self, config: &ConnectionConfig) -> Result<String, AppError> {
        let url = Self::build_connection_url(config);
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(format!("MySQL connection failed: {}", e)))?;
        let conn_id = uuid::Uuid::new_v4().to_string();
        let mut connections = self.connections.write().await;
        connections.insert(conn_id.clone(), pool);
        Ok(conn_id)
    }

    async fn disconnect(&self, conn_id: &str) -> Result<(), AppError> {
        let mut connections = self.connections.write().await;
        if let Some(pool) = connections.remove(conn_id) {
            pool.close().await;
        }
        Ok(())
    }

    async fn test_connection(&self, config: &ConnectionConfig) -> Result<bool, AppError> {
        let url = Self::build_connection_url(config);
        let pool = MySqlPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(format!("MySQL test connection failed: {}", e)))?;
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| AppError::Connection(format!("MySQL ping failed: {}", e)))?;
        pool.close().await;
        Ok(true)
    }

    async fn execute(&self, conn_id: &str, sql: &str) -> Result<QueryResult, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let start = std::time::Instant::now();
        let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Query execution failed: {}", e)))?;
        let execution_time_ms = start.elapsed().as_millis() as u64;

        let columns: Vec<ColumnDef> = if let Some(first_row) = rows.first() {
            first_row.columns()
                .iter()
                .map(|col| ColumnDef {
                    name: col.name().to_string(),
                    data_type: col.type_info().to_string(),
                    nullable: true,
                })
                .collect()
        } else {
            Vec::new()
        };

        let result_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    let value: Option<String> = row.try_get(i).ok();
                    map.insert(
                        col.name.clone(),
                        match value {
                            Some(v) => serde_json::Value::String(v),
                            None => serde_json::Value::Null,
                        },
                    );
                }
                serde_json::Value::Object(map)
            })
            .collect();
        Ok(QueryResult {
            columns,
            rows: result_rows,
            affected_rows: 0,
            execution_time_ms,
        })
    }

    async fn metadata(&self, conn_id: &str) -> Result<DatabaseMetadata, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let databases: Vec<(String,)> = sqlx::query_as("SHOW DATABASES")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch databases: {}", e)))?;
        let catalogs = databases.iter()
            .map(|(name,)| CatalogInfo { name: name.clone() })
            .collect();
        let tables: Vec<(String, String, Option<String>, Option<i64>)> = sqlx::query_as(
            "SELECT TABLE_NAME, TABLE_TYPE, TABLE_COMMENT, TABLE_ROWS FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = DATABASE()"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch tables: {}", e)))?;
        let table_infos = tables.iter()
            .map(|(name, table_type, comment, row_count)| TableInfo {
                catalog: None,
                schema: None,
                name: name.clone(),
                table_type: table_type.clone(),
                comment: comment.clone(),
                row_count: *row_count,
            })
            .collect();
        let views: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT TABLE_NAME, VIEW_DEFINITION FROM INFORMATION_SCHEMA.VIEWS WHERE TABLE_SCHEMA = DATABASE()"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch views: {}", e)))?;
        let view_infos = views.iter()
            .map(|(name, definition)| ViewInfo {
                catalog: None,
                schema: None,
                name: name.clone(),
                definition: definition.clone(),
            })
            .collect();
        Ok(DatabaseMetadata {
            catalogs,
            schemas: Vec::new(),
            tables: table_infos,
            views: view_infos,
        })
    }

    async fn table_schema(&self, conn_id: &str, table: &str) -> Result<TableSchema, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let columns: Vec<(String, String, Option<String>, String, String, Option<String>, i32, Option<i64>, Option<i32>, Option<i32>)> = sqlx::query_as(
            "SELECT COLUMN_NAME, COLUMN_TYPE, COLUMN_DEFAULT, IS_NULLABLE, COLUMN_KEY, COLUMN_COMMENT, ORDINAL_POSITION, CHARACTER_MAXIMUM_LENGTH, NUMERIC_PRECISION, NUMERIC_SCALE FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION"
        )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch columns: {}", e)))?;
        let column_infos = columns.iter()
            .map(|(name, data_type, default, nullable, key, comment, pos, max_len, precision, scale)| ColumnInfo {
                name: name.clone(),
                data_type: data_type.clone(),
                nullable: nullable == "YES",
                default_value: default.clone(),
                is_primary_key: key == "PRI",
                is_auto_increment: key == "PRI",
                comment: comment.clone(),
                ordinal_position: *pos,
                max_length: *max_len,
                precision: *precision,
                scale: *scale,
            })
            .collect();
        let indexes: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT INDEX_NAME, COLUMN_NAME, NON_UNIQUE, INDEX_TYPE FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY INDEX_NAME, SEQ_IN_INDEX"
        )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch indexes: {}", e)))?;
        let mut index_map: HashMap<String, IndexInfo> = HashMap::new();
        for (idx_name, col_name, non_unique, idx_type) in &indexes {
            let entry = index_map.entry(idx_name.clone()).or_insert_with(|| IndexInfo {
                name: idx_name.clone(),
                columns: Vec::new(),
                is_unique: non_unique == "0",
                is_primary: idx_name == "PRIMARY",
                index_type: Some(idx_type.clone()),
            });
            entry.columns.push(col_name.clone());
        }
        Ok(TableSchema {
            table_name: table.to_string(),
            columns: column_infos,
            indexes: index_map.into_values().collect(),
            constraints: Vec::new(),
        })
    }

    async fn explain(&self, conn_id: &str, sql: &str) -> Result<ExplainResult, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let explain_sql = format!("EXPLAIN {}", sql);
        let rows: Vec<sqlx::mysql::MySqlRow> = sqlx::query(&explain_sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("EXPLAIN failed: {}", e)))?;
        let plan: String = rows.iter()
            .map(|row| format!("{:?}", row))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(ExplainResult { plan, formatted: None, cost: None })
    }

    async fn export(&self, _conn_id: &str, _config: &ExportConfig) -> Result<ExportResult, AppError> {
        Err(AppError::Plugin("Export not yet implemented".to_string()))
    }

    async fn import(&self, _conn_id: &str, _config: &ImportConfig) -> Result<ImportResult, AppError> {
        Err(AppError::Plugin("Import not yet implemented".to_string()))
    }

    fn dialect(&self) -> SqlDialect {
        SqlDialect::MySQL
    }
}
