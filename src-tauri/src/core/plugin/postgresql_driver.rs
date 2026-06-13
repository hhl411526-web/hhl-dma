use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Postgres, Row, Column};
use crate::core::plugin::driver::DatabaseDriver;
use crate::core::types::*;
use crate::errors::AppError;

pub struct PostgresqlDriver {
    connections: RwLock<HashMap<String, sqlx::Pool<Postgres>>>,
}

impl PostgresqlDriver {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    fn build_connection_url(config: &ConnectionConfig) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username,
            urlencoding::encode(&config.password),
            config.host,
            config.port,
            config.database
        )
    }
}

#[async_trait]
impl DatabaseDriver for PostgresqlDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "PostgreSQL Driver".to_string(),
            version: "0.1.0".to_string(),
            db_type: DatabaseType::PostgreSQL,
            description: "PostgreSQL database driver".to_string(),
        }
    }

    async fn connect(&self, config: &ConnectionConfig) -> Result<String, AppError> {
        let url = Self::build_connection_url(config);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(format!("PostgreSQL connection failed: {}", e)))?;
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
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(format!("PostgreSQL test connection failed: {}", e)))?;
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| AppError::Connection(format!("PostgreSQL ping failed: {}", e)))?;
        pool.close().await;
        Ok(true)
    }

    async fn execute(&self, conn_id: &str, sql: &str) -> Result<QueryResult, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;
        let start = std::time::Instant::now();
        let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(sql)
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

        let databases: Vec<(String,)> = sqlx::query_as(
            "SELECT datname FROM pg_database WHERE datistemplate = false"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch databases: {}", e)))?;
        let catalogs = databases.iter()
            .map(|(name,)| CatalogInfo { name: name.clone() })
            .collect();

        let schemas: Vec<(String,)> = sqlx::query_as(
            "SELECT schema_name FROM information_schema.schemata"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch schemas: {}", e)))?;
        let schema_infos = schemas.iter()
            .map(|(name,)| SchemaInfo {
                catalog: None,
                name: name.clone(),
            })
            .collect();

        let tables: Vec<(String, String)> = sqlx::query_as(
            "SELECT table_name, table_type FROM information_schema.tables WHERE table_schema = 'public'"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch tables: {}", e)))?;
        let table_infos = tables.iter()
            .map(|(name, table_type)| TableInfo {
                catalog: None,
                schema: Some("public".to_string()),
                name: name.clone(),
                table_type: table_type.clone(),
                comment: None,
                row_count: None,
            })
            .collect();

        let views: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT table_name, view_definition FROM information_schema.views WHERE table_schema = 'public'"
        )
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch views: {}", e)))?;
        let view_infos = views.iter()
            .map(|(name, definition)| ViewInfo {
                catalog: None,
                schema: Some("public".to_string()),
                name: name.clone(),
                definition: definition.clone(),
            })
            .collect();

        Ok(DatabaseMetadata {
            catalogs,
            schemas: schema_infos,
            tables: table_infos,
            views: view_infos,
        })
    }

    async fn table_schema(&self, conn_id: &str, table: &str) -> Result<TableSchema, AppError> {
        let connections = self.connections.read().await;
        let pool = connections.get(conn_id)
            .ok_or_else(|| AppError::Connection(format!("Connection {} not found", conn_id)))?;

        let columns: Vec<(String, String, Option<String>, String, Option<String>, Option<i64>, Option<i32>, Option<i32>)> = sqlx::query_as(
            "SELECT column_name, data_type, column_default, is_nullable, column_name as col_key, character_maximum_length, numeric_precision, numeric_scale FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position"
        )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch columns: {}", e)))?;

        // Determine primary key columns
        let pk_columns: Vec<String> = sqlx::query(
            "SELECT kcu.column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_name = $1 AND tc.table_schema = 'public'"
        )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch primary keys: {}", e)))?
            .iter()
            .map(|row| row.get(0))
            .collect();

        let column_infos = columns.iter()
            .map(|(name, data_type, default, nullable, _col_key, max_len, precision, scale)| ColumnInfo {
                name: name.clone(),
                data_type: data_type.clone(),
                nullable: nullable == "YES",
                default_value: default.clone(),
                is_primary_key: pk_columns.contains(name),
                is_auto_increment: default.as_ref().map_or(false, |d| d.contains("nextval")),
                comment: None,
                ordinal_position: 0,
                max_length: *max_len,
                precision: *precision,
                scale: *scale,
            })
            .collect();

        let indexes: Vec<(String, String)> = sqlx::query_as(
            "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = $1"
        )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch indexes: {}", e)))?;

        let mut index_map: HashMap<String, IndexInfo> = HashMap::new();
        for (idx_name, idx_def) in &indexes {
            let is_primary = idx_name.starts_with("pk_") || idx_def.contains("PRIMARY KEY");
            let is_unique = idx_def.contains("UNIQUE") || is_primary;
            // Extract column names from index definition
            let columns_str = idx_def.split('(').nth(1)
                .and_then(|s| s.split(')').next())
                .unwrap_or("");
            let idx_columns: Vec<String> = columns_str
                .split(',')
                .map(|c| c.trim().trim_matches('"').to_string())
                .filter(|c| !c.is_empty())
                .collect();
            let entry = index_map.entry(idx_name.clone()).or_insert_with(|| IndexInfo {
                name: idx_name.clone(),
                columns: Vec::new(),
                is_unique,
                is_primary,
                index_type: Some("B-Tree".to_string()),
            });
            entry.columns.extend(idx_columns);
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
        let explain_sql = format!("EXPLAIN ANALYZE {}", sql);
        let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(&explain_sql)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Query(format!("EXPLAIN ANALYZE failed: {}", e)))?;
        let plan: String = rows.iter()
            .map(|row| {
                let line: String = row.try_get(0).unwrap_or_default();
                line
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
        SqlDialect::PostgreSQL
    }
}
