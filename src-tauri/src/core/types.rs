use serde::{Deserialize, Serialize};
use chrono::DateTime;
use chrono::Utc;

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub schema: Option<String>,
    pub options: std::collections::HashMap<String, String>,
    pub group: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 支持的数据库类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    SQLite,
    Oracle,
    SQLServer,
    DM,
    KingbaseES,
    Oscar,
    GBase,
    ODBC,
}

/// 数据库元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    pub catalogs: Vec<CatalogInfo>,
    pub schemas: Vec<SchemaInfo>,
    pub tables: Vec<TableInfo>,
    pub views: Vec<ViewInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub catalog: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub catalog: Option<String>,
    pub schema: Option<String>,
    pub name: String,
    pub table_type: String,
    pub comment: Option<String>,
    pub row_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInfo {
    pub catalog: Option<String>,
    pub schema: Option<String>,
    pub name: String,
    pub definition: Option<String>,
}

/// 表结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub constraints: Vec<ConstraintInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_auto_increment: bool,
    pub comment: Option<String>,
    pub ordinal_position: i32,
    pub max_length: Option<i64>,
    pub precision: Option<i32>,
    pub scale: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintInfo {
    pub name: String,
    pub constraint_type: String,
    pub columns: Vec<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Option<Vec<String>>,
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<serde_json::Value>,
    pub affected_rows: u64,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// 执行计划结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub plan: String,
    pub formatted: Option<String>,
    pub cost: Option<f64>,
}

/// 导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub table: Option<String>,
    pub query: Option<String>,
    pub output_path: String,
    pub include_headers: bool,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    SQL,
}

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub rows_exported: u64,
    pub file_path: String,
    pub file_size_bytes: u64,
}

/// 导入配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConfig {
    pub format: ImportFormat,
    pub table: String,
    pub file_path: String,
    pub has_headers: bool,
    pub encoding: String,
    pub on_conflict: ConflictStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportFormat {
    CSV,
    JSON,
    SQL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictStrategy {
    Skip,
    Update,
    Error,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub rows_imported: u64,
    pub rows_skipped: u64,
    pub errors: Vec<String>,
}

/// SQL 方言
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SqlDialect {
    MySQL,
    PostgreSQL,
    SQLite,
    Oracle,
    SQLServer,
    DM,
    Generic,
}

/// 驱动信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub name: String,
    pub version: String,
    pub db_type: DatabaseType,
    pub description: String,
}
