use async_trait::async_trait;
use crate::core::types::*;
use crate::errors::AppError;

/// 所有数据库插件必须实现的核心接口
#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    /// 插件元信息
    fn info(&self) -> DriverInfo;

    /// 建立连接，返回连接ID
    async fn connect(&self, config: &ConnectionConfig) -> Result<String, AppError>;

    /// 断开连接
    async fn disconnect(&self, conn_id: &str) -> Result<(), AppError>;

    /// 测试连接
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<bool, AppError>;

    /// 执行 SQL
    async fn execute(&self, conn_id: &str, sql: &str) -> Result<QueryResult, AppError>;

    /// 获取元数据
    async fn metadata(&self, conn_id: &str) -> Result<DatabaseMetadata, AppError>;

    /// 获取表结构
    async fn table_schema(&self, conn_id: &str, table: &str) -> Result<TableSchema, AppError>;

    /// 获取执行计划
    async fn explain(&self, conn_id: &str, sql: &str) -> Result<ExplainResult, AppError>;

    /// 导出数据
    async fn export(&self, conn_id: &str, config: &ExportConfig) -> Result<ExportResult, AppError>;

    /// 导入数据
    async fn import(&self, conn_id: &str, config: &ImportConfig) -> Result<ImportResult, AppError>;

    /// 获取 SQL 方言
    fn dialect(&self) -> SqlDialect;
}
