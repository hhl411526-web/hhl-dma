use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Sqlite, Row, Column};
use crate::core::plugin::driver::DatabaseDriver;
use crate::core::types::*;
use crate::errors::AppError;

pub struct SqliteDriver {
    connections: RwLock<HashMap<String, sqlx::Pool<Sqlite>>>,
}

impl SqliteDriver {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    fn build_connection_url(config: &ConnectionConfig) -> String {
        format!("sqlite:{}", config.database)
    }
}

#[async_trait]
impl DatabaseDriver for SqliteDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "SQLite Driver".to_string(),
            version: "0.1.0".to_string(),
            db_type: DatabaseType::SQLite,
            description: "SQLite database driver".to_string(),
        }
    }

    async fn connect(&self, config: &ConnectionConfig) -> Result<String, AppError> {
        let url = Self::build_connection_url(config);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(format!("SQLite connection failed: {}", e)))?;
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
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(format!("SQLite test connection failed: {}", e)))?;
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| AppError::Connection(format!("SQLite ping failed: {}", e)))?;
        pool.close().await;
        Ok(true)
    }

    async fn execute(&self, conn_id: &str, sql: &str) -> Result<QueryResult, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let start = std::time::Instant::now();
        let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(sql)
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

        // SQLite has a single catalog/database
        let catalogs = vec![CatalogInfo { name: "main".to_string() }];

        let tables: Vec<(String, String)> = sqlx::query_as(
            "SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch tables: {}", e)))?;

        let mut table_infos = Vec::new();
        let mut view_infos = Vec::new();

        for (name, obj_type) in &tables {
            if obj_type == "table" {
                table_infos.push(TableInfo {
                    catalog: Some("main".to_string()),
                    schema: None,
                    name: name.clone(),
                    table_type: "BASE TABLE".to_string(),
                    comment: None,
                    row_count: None,
                });
            } else if obj_type == "view" {
                // Get view definition
                let def: Option<(String,)> = sqlx::query_as(
                    "SELECT sql FROM sqlite_master WHERE type = 'view' AND name = ?"
                )
                    .bind(name)
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| AppError::Query(format!("Failed to fetch view definition: {}", e)))?;

                view_infos.push(ViewInfo {
                    catalog: Some("main".to_string()),
                    schema: None,
                    name: name.clone(),
                    definition: def.map(|(sql,)| sql),
                });
            }
        }

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

        // Use PRAGMA table_info to get column information
        let pragma_sql = format!("PRAGMA table_info(\"{}\")", table.replace('"', "\"\""));
        let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(&pragma_sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch columns: {}", e)))?;

        let column_infos: Vec<ColumnInfo> = rows.iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let data_type: String = row.try_get("type").unwrap_or_default();
                let notnull: i32 = row.try_get("notnull").unwrap_or(0);
                let default_value: Option<String> = row.try_get("dflt_value").unwrap_or(None);
                let pk: i32 = row.try_get("pk").unwrap_or(0);
                let cid: i32 = row.try_get("cid").unwrap_or(0);

                let is_auto_increment = pk > 0 && data_type.to_uppercase().contains("INTEGER");

                ColumnInfo {
                    name,
                    data_type,
                    nullable: notnull == 0,
                    default_value,
                    is_primary_key: pk > 0,
                    is_auto_increment,
                    comment: None,
                    ordinal_position: cid,
                    max_length: None,
                    precision: None,
                    scale: None,
                }
            })
            .collect();

        // Get indexes
        let index_list_sql = format!("PRAGMA index_list(\"{}\")", table.replace('"', "\"\""));
        let index_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(&index_list_sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch index list: {}", e)))?;

        let mut index_infos = Vec::new();
        for idx_row in &index_rows {
            let idx_name: String = idx_row.try_get("name").unwrap_or_default();
            let unique: i32 = idx_row.try_get("unique").unwrap_or(0);
            let origin: String = idx_row.try_get("origin").unwrap_or_default();

            let idx_info_sql = format!("PRAGMA index_info(\"{}\")", idx_name.replace('"', "\"\""));
            let idx_info_rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(&idx_info_sql)
                .fetch_all(pool)
                .await
                .map_err(|e| AppError::Query(format!("Failed to fetch index info: {}", e)))?;

            let idx_columns: Vec<String> = idx_info_rows.iter()
                .map(|r| r.try_get("name").unwrap_or_default())
                .collect();

            index_infos.push(IndexInfo {
                name: idx_name,
                columns: idx_columns,
                is_unique: unique != 0,
                is_primary: origin == "c",
                index_type: None,
            });
        }

        Ok(TableSchema {
            table_name: table.to_string(),
            columns: column_infos,
            indexes: index_infos,
            constraints: Vec::new(),
        })
    }

    async fn explain(&self, conn_id: &str, sql: &str) -> Result<ExplainResult, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let explain_sql = format!("EXPLAIN QUERY PLAN {}", sql);
        let rows: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(&explain_sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("EXPLAIN QUERY PLAN failed: {}", e)))?;
        let plan: String = rows.iter()
            .map(|row| {
                let detail: String = row.try_get("detail").unwrap_or_default();
                detail
            })
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
        SqlDialect::SQLite
    }
}
